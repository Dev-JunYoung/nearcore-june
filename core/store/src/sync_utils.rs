use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use std::ops::{Deref, DerefMut};
use std::sync::{Condvar, Mutex, MutexGuard};

const POISONED_LOCK_ERR: &str = "The lock was poisoned.";

/// A convenience wrapper around a Mutex and a Condvar.
///
/// It enables blocking while waiting for the underlying value to be updated.
/// The implementation ensures that any modification results in all blocked
/// threads being notified.
pub(crate) struct Monitor<T> {
    cvar: Condvar,
    mutex: Mutex<T>,
}

pub(crate) struct MonitorReadGuard<'a, T> {
    guard: MutexGuard<'a, T>,
}

pub(crate) struct MonitorWriteGuard<'a, T> {
    guard: MutexGuard<'a, T>,
    cvar: &'a Condvar,
}

impl<T> Monitor<T> {
    pub fn new(t: T) -> Self {
print_file_path_and_function_name!();

        Self { mutex: Mutex::new(t), cvar: Condvar::new() }
    }

    pub fn lock(&self) -> MonitorReadGuard<'_, T> {
print_file_path_and_function_name!();

        let guard = self.mutex.lock().expect(POISONED_LOCK_ERR);
        MonitorReadGuard { guard }
    }

    pub fn lock_mut(&self) -> MonitorWriteGuard<'_, T> {
print_file_path_and_function_name!();

        let guard = self.mutex.lock().expect(POISONED_LOCK_ERR);
        MonitorWriteGuard { guard, cvar: &self.cvar }
    }

    pub fn wait<'a>(&'a self, guard: MonitorReadGuard<'a, T>) -> MonitorReadGuard<'a, T> {
print_file_path_and_function_name!();

        let guard = self.cvar.wait(guard.guard).expect(POISONED_LOCK_ERR);
        MonitorReadGuard { guard }
    }
}

impl<T> Deref for MonitorReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
print_file_path_and_function_name!();

        self.guard.deref()
    }
}

impl<T> Deref for MonitorWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
print_file_path_and_function_name!();

        self.guard.deref()
    }
}

impl<T> DerefMut for MonitorWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
print_file_path_and_function_name!();

        self.guard.deref_mut()
    }
}

impl<T> Drop for MonitorWriteGuard<'_, T> {
    fn drop(&mut self) {
print_file_path_and_function_name!();

        self.cvar.notify_all();
    }
}
