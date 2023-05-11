use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use crate::messaging::Sender;

pub(crate) struct TimedComponent {
    buffered_messages: Vec<String>,
    message_sender: Sender<Vec<String>>,
}

/// Mimics a component that has a specific function that is supposed to be
/// triggered by a timer.
impl TimedComponent {
    pub fn new(message_sender: Sender<Vec<String>>) -> Self {
print_file_path_and_function_name!();

        Self { buffered_messages: vec![], message_sender }
    }

    pub fn send_message(&mut self, msg: String) {
print_file_path_and_function_name!();

        self.buffered_messages.push(msg);
    }

    /// This is supposed to be triggered by a timer so it flushes the
    /// messages every tick.
    pub fn flush(&mut self) {
print_file_path_and_function_name!();

        if self.buffered_messages.is_empty() {
            return;
        }
        self.message_sender.send(self.buffered_messages.clone());
        self.buffered_messages.clear();
    }
}
