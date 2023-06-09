// use blockbuster::Utc;
// use blockbuster::DepthGuard;
// use blockbuster::DEPTH_COUNTER;
// use blockbuster::TOTAL_COUNTER;
// use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;
//

macro_rules! to_str {
    ($v:expr) => {
        ::std::convert::identity::<String>($v.into()).as_str()
    };
}

macro_rules! common_conversions {
    ($ty:ty, $what:literal) => {
        impl ::std::convert::TryFrom<String> for $ty {
            type Error = ();

            fn try_from(value: String) -> Result<Self, ()> {
// print_file_path_and_function_name!();

                Self::try_from(value.as_str())
            }
        }

        impl From<$ty> for String {
            fn from(v: $ty) -> String {
// print_file_path_and_function_name!();

                bs58::encode(v).into_string()
            }
        }

        impl From<&$ty> for String {
            fn from(v: &$ty) -> String {
// print_file_path_and_function_name!();

                bs58::encode(v).into_string()
            }
        }

        impl ::std::fmt::Debug for $ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
// print_file_path_and_function_name!();

                f.write_str(to_str!(self))
            }
        }

        impl ::std::fmt::Display for $ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
// print_file_path_and_function_name!();

                f.write_str(to_str!(self))
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $ty {
            fn deserialize<D: ::serde::Deserializer<'de>>(
                deserializer: D,
            ) -> Result<Self, D::Error> {
// print_file_path_and_function_name!();

                let s = <&str as ::serde::Deserialize<'de>>::deserialize(deserializer)?;
                <Self as ::std::convert::TryFrom<&str>>::try_from(s).map_err(|_| {
                    <D::Error as ::serde::de::Error>::invalid_value(
                        ::serde::de::Unexpected::Str(s),
                        &concat!("a valid ", $what),
                    )
                })
            }
        }

        impl ::serde::Serialize for $ty {
            fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
// print_file_path_and_function_name!();

                serializer.serialize_str(to_str!(self))
            }
        }
    };
}

macro_rules! common_conversions_fixed {
    ($ty:ty, $l:literal, $bytes:expr, $what:literal) => {
        impl AsRef<[u8; $l]> for $ty {
            fn as_ref(&self) -> &[u8; $l] {
// print_file_path_and_function_name!();

                ::std::convert::identity::<fn(&$ty) -> &[u8; $l]>($bytes)(self)
            }
        }

        impl AsRef<[u8]> for $ty {
            fn as_ref(&self) -> &[u8] {
// print_file_path_and_function_name!();

                <Self as AsRef<[u8; $l]>>::as_ref(self)
            }
        }

        impl From<$ty> for [u8; $l] {
            fn from(v: $ty) -> [u8; $l] {
// print_file_path_and_function_name!();

                *v.as_ref()
            }
        }

        impl From<&$ty> for [u8; $l] {
            fn from(v: &$ty) -> [u8; $l] {
// print_file_path_and_function_name!();

                *v.as_ref()
            }
        }

        impl ::std::convert::TryFrom<&str> for $ty {
            type Error = ();

            fn try_from(value: &str) -> Result<Self, ()> {
// print_file_path_and_function_name!();

                let mut buf = [0; $l];
                match bs58::decode(value).into(&mut buf[..]) {
                    Ok($l) => Self::try_from(&buf).or(Err(())),
                    _ => Err(()),
                }
            }
        }

        common_conversions!($ty, $what);
    };
}

macro_rules! eq {
    ($ty:ty, $e:expr) => {
        impl PartialEq for $ty {
            fn eq(&self, other: &Self) -> bool {
// print_file_path_and_function_name!();

                ::std::convert::identity::<fn(&Self, &Self) -> bool>($e)(self, other)
            }
        }

        impl Eq for $ty {}
    };
}

macro_rules! value_type {
    ($vis:vis, $ty:ident, $l:literal, $what:literal) => {
        #[derive(Copy, Clone, Eq, PartialEq, borsh::BorshDeserialize, borsh::BorshSerialize)]
        $vis struct $ty(pub [u8; $l]);

        impl AsMut<[u8; $l]> for $ty {
            fn as_mut(&mut self) -> &mut [u8; $l] {
// print_file_path_and_function_name!();

                &mut self.0
            }
        }

        impl AsMut<[u8]> for $ty {
            fn as_mut(&mut self) -> &mut [u8] {
// print_file_path_and_function_name!();

                &mut self.0[..]
            }
        }

        impl From<&[u8; $l]> for $ty {
            fn from(value: &[u8; $l]) -> Self {
// print_file_path_and_function_name!();

                Self(*value)
            }
        }

        common_conversions_fixed!($ty, $l, |s| &s.0, $what);
    };
}
