// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! Compact representation of `Option<T>` for types with a reserved value.
//!
//! Small types are often used in tables and linked lists where an
//! `Option<T>` is needed. Unfortunately, that would double the size of the tables
//! because `Option<T>` is twice as big as `T`.
//!
//! This module provides a `PackedOption<T>` for types that have a reserved value that can be used
//! to represent `None`.


use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use crate::lib::std::fmt;
use crate::lib::std::mem;

/// Types that have a reserved value which can't be created any other way.
pub trait ReservedValue {
    /// Create an instance of the reserved value.
    fn reserved_value() -> Self;
    /// Checks whether value is the reserved one.
    fn is_reserved_value(&self) -> bool;
}

/// Packed representation of `Option<T>`.
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct PackedOption<T: ReservedValue>(T);

impl<T: ReservedValue> PackedOption<T> {
    /// Returns `true` if the packed option is a `None` value.
    pub fn is_none(&self) -> bool {
print_file_path_and_function_name!();

        self.0.is_reserved_value()
    }

    /// Returns `true` if the packed option is a `Some` value.
    pub fn is_some(&self) -> bool {
print_file_path_and_function_name!();

        !self.0.is_reserved_value()
    }

    /// Expand the packed option into a normal `Option`.
    pub fn expand(self) -> Option<T> {
print_file_path_and_function_name!();

        if self.is_none() {
            None
        } else {
            Some(self.0)
        }
    }

    /// Maps a `PackedOption<T>` to `Option<U>` by applying a function to a contained value.
    pub fn map<U, F>(self, f: F) -> Option<U>
    where
        F: FnOnce(T) -> U,
    {
print_file_path_and_function_name!();

        self.expand().map(f)
    }

    /// Unwrap a packed `Some` value or panic.
    pub fn unwrap(self) -> T {
print_file_path_and_function_name!();

        self.expand().unwrap()
    }

    /// Unwrap a packed `Some` value or panic.
    pub fn expect(self, msg: &str) -> T {
print_file_path_and_function_name!();

        self.expand().expect(msg)
    }

    /// Takes the value out of the packed option, leaving a `None` in its place.
    pub fn take(&mut self) -> Option<T> {
print_file_path_and_function_name!();

        mem::replace(self, None.into()).expand()
    }
}

impl<T: ReservedValue> Default for PackedOption<T> {
    /// Create a default packed option representing `None`.
    fn default() -> Self {
print_file_path_and_function_name!();

        Self(T::reserved_value())
    }
}

impl<T: ReservedValue> From<T> for PackedOption<T> {
    /// Convert `t` into a packed `Some(x)`.
    fn from(t: T) -> Self {
print_file_path_and_function_name!();

        debug_assert!(!t.is_reserved_value(), "Can't make a PackedOption from the reserved value.");
        Self(t)
    }
}

impl<T: ReservedValue> From<Option<T>> for PackedOption<T> {
    /// Convert an option into its packed equivalent.
    fn from(opt: Option<T>) -> Self {
print_file_path_and_function_name!();

        match opt {
            None => Self::default(),
            Some(t) => t.into(),
        }
    }
}

impl<T: ReservedValue> Into<Option<T>> for PackedOption<T> {
    fn into(self) -> Option<T> {
print_file_path_and_function_name!();

        self.expand()
    }
}

impl<T> fmt::Debug for PackedOption<T>
where
    T: ReservedValue + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
print_file_path_and_function_name!();

        if self.is_none() {
            write!(f, "None")
        } else {
            write!(f, "Some({:?})", self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Dummy entity class, with no Copy or Clone.
    #[derive(Debug, PartialEq, Eq)]
    struct NoC(u32);

    impl ReservedValue for NoC {
        fn reserved_value() -> Self {
            Self(13)
        }

        fn is_reserved_value(&self) -> bool {
            self.0 == 13
        }
    }

    #[test]
    fn moves() {
        let x = NoC(3);
        let somex: PackedOption<NoC> = x.into();
        assert!(!somex.is_none());
        assert_eq!(somex.expand(), Some(NoC(3)));

        let none: PackedOption<NoC> = None.into();
        assert!(none.is_none());
        assert_eq!(none.expand(), None);
    }

    // Dummy entity class, with Copy.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Ent(u32);

    impl ReservedValue for Ent {
        fn reserved_value() -> Self {
            Self(13)
        }

        fn is_reserved_value(&self) -> bool {
            self.0 == 13
        }
    }

    #[test]
    fn copies() {
        let x = Ent(2);
        let some: PackedOption<Ent> = x.into();
        let some2: Option<Ent> = x.into();
        assert_eq!(some.expand(), some2);
        assert_eq!(some, x.into());
    }
}
