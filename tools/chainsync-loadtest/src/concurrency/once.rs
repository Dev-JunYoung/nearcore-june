use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use parking_lot::RwLock;
use std::future::Future;

fn is_send<T: Send>() {}
fn is_sync<T: Sync>() {}

#[allow(dead_code)]
fn test<T: Clone + Send + Sync>() {
print_file_path_and_function_name!();

    is_send::<Once<T>>();
    is_sync::<Once<T>>();
}

// Once is a synchronization primitive, which stores a single value.
// This value can be set at most once, and multiple consumers are
// allowed to wait for that value.
pub struct Once<T> {
    value: RwLock<Option<T>>,
    notify: tokio::sync::Notify,
}

impl<T: Clone + Send + Sync> Once<T> {
    pub fn new() -> Once<T> {
print_file_path_and_function_name!();

        return Once { value: RwLock::new(None), notify: tokio::sync::Notify::new() };
    }

    // set() sets the value of Once to x.
    // Returns x back to the caller, in case Once has already been set.
    pub fn set(&self, x: T) -> Result<(), T> {
print_file_path_and_function_name!();

        let mut v = self.value.write();
        if v.is_some() {
            return Err(x);
        }
        *v = Some(x);
        self.notify.notify_waiters();
        drop(v);
        return Ok(());
    }

    // get() gets a clone of the value, or returns None if not set yet.
    pub fn get(&self) -> Option<T> {
print_file_path_and_function_name!();

        self.value.read().clone()
    }

    // wait() waits for Once to be set, then returns a clone of the value.
    pub fn wait(&self) -> impl Future<Output = T> + Send + '_ {
print_file_path_and_function_name!();

        let l = self.value.read();
        let v = (*l).clone();
        let n = self.notify.notified();
        drop(l);
        async move {
            if let Some(v) = v {
                return v;
            }
            n.await;
            return self.get().unwrap();
        }
    }
}
