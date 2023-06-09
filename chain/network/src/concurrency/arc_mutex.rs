use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use arc_swap::ArcSwap;
use std::sync::{Arc, Mutex};

/// Mutex which only synchronizes on writes.
/// Reads always succeed and return the latest written version.
pub struct ArcMutex<T> {
    value: ArcSwap<T>,
    mutex: Mutex<()>,
}

impl<T: Clone> ArcMutex<T> {
    pub fn new(v: T) -> Self {
print_file_path_and_function_name!();

        Self { value: ArcSwap::new(Arc::new(v)), mutex: Mutex::new(()) }
    }

    /// Loads the last value stored. Non-blocking.
    pub fn load(&self) -> Arc<T> {
print_file_path_and_function_name!();

        self.value.load_full()
    }

    /// Atomic update of the value. Blocking.
    /// Note that `T -> (R,T)` is a state monad.
    /// State monad is a function which takes the old state and
    /// returns the new state + additional result value.
    pub fn update<R>(&self, f: impl FnOnce(T) -> (R, T)) -> R {
print_file_path_and_function_name!();

        let _guard = self.mutex.lock().unwrap();
        let (res, val) = f(self.value.load().as_ref().clone());
        self.value.store(Arc::new(val));
        res
    }

    /// Atomic update of the value. Value is not modified if an error is returned. Blocking.
    /// Note that `T -> Result<(R,T),E>` is a state monad transformer applied to the exception
    /// monad.
    pub fn try_update<R, E>(&self, f: impl FnOnce(T) -> Result<(R, T), E>) -> Result<R, E> {
print_file_path_and_function_name!();

        let _guard = self.mutex.lock().unwrap();
        match f(self.value.load().as_ref().clone()) {
            Ok((res, val)) => {
                self.value.store(Arc::new(val));
                Ok(res)
            }
            Err(e) => Err(e),
        }
    }
}
