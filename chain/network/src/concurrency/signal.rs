use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use std::sync::Arc;

#[derive(Clone)]
pub(super) struct Once(Arc<tokio::sync::Semaphore>);

impl Once {
    pub fn new() -> Self {
print_file_path_and_function_name!();

        Self(Arc::new(tokio::sync::Semaphore::new(0)))
    }

    /// Sends the signal, waking all tasks awaiting for recv().
    ///
    /// After this call recv().await will always return immediately.
    /// After this call any subsequent call to send() is a noop.
    pub fn send(&self) {
print_file_path_and_function_name!();

        self.0.close();
    }

    /// recv() waits for the first call to send().
    ///
    /// Cancellable.
    pub async fn recv(&self) {
print_file_path_and_function_name!();

        // We await for the underlying semaphore to get closed.
        // This is the only possible outcome, because we never add
        // any permits to the semaphore.
        let res = self.0.acquire().await;
        debug_assert!(res.is_err());
    }

    /// Checks if send() was already called.
    pub fn try_recv(&self) -> bool {
print_file_path_and_function_name!();

        self.0.is_closed()
    }
}
