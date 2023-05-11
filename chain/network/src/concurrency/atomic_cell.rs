use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use std::sync::Mutex;

// AtomicCell narrows down a Mutex API to load/store calls.
pub(crate) struct AtomicCell<T>(Mutex<T>);

impl<T: Clone> AtomicCell<T> {
    pub fn new(v: T) -> Self {
print_file_path_and_function_name!();

        Self(Mutex::new(v))
    }
    pub fn load(&self) -> T {
print_file_path_and_function_name!();

        self.0.lock().unwrap().clone()
    }
    pub fn store(&self, v: T) {
print_file_path_and_function_name!();

        *self.0.lock().unwrap() = v;
    }
}
