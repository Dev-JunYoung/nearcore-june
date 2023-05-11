use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use super::event_handler::{LoopEventHandler, LoopEventHandlerImpl, LoopHandlerContext};

/// Event handler that handles a specific single instance in a multi-instance
/// setup.
///
/// To convert a single-instance handler to a multi-instance handler
/// (for one instance), use handler.for_index(index).
pub(crate) struct IndexedLoopEventHandler<Data: 'static, Event: 'static> {
    pub(crate) inner: LoopEventHandler<Data, Event>,
    pub(crate) index: usize,
}

impl<Data, Event> LoopEventHandlerImpl<Vec<Data>, (usize, Event)>
    for IndexedLoopEventHandler<Data, Event>
{
    fn init(&mut self, context: LoopHandlerContext<(usize, Event)>) {
print_file_path_and_function_name!();

        self.inner.init(LoopHandlerContext {
            sender: context.sender.for_index(self.index),
            clock: context.clock,
        })
    }

    fn handle(
        &mut self,
        event: (usize, Event),
        data: &mut Vec<Data>,
    ) -> Result<(), (usize, Event)> {
print_file_path_and_function_name!();

        if event.0 == self.index {
            self.inner.handle(event.1, &mut data[self.index]).map_err(|event| (self.index, event))
        } else {
            Err(event)
        }
    }

    fn try_drop(&self, event: (usize, Event)) -> Result<(), (usize, Event)> {
print_file_path_and_function_name!();

        if event.0 == self.index {
            self.inner.try_drop(event.1).map_err(|event| (self.index, event))
        } else {
            Err(event)
        }
    }
}
