use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use futures::future::{BoxFuture, Future, FutureExt};

/// Boxed asynchronous function. In rust asynchronous functions
/// are just regular functions which return a Future.
/// This is a convenience alias to express a
pub type BoxAsyncFn<'a, Arg, Res> = Box<dyn 'a + FnOnce(Arg) -> BoxFuture<'a, Res>>;

/// AsyncFn trait represents asynchronous functions which can be boxed.
pub trait AsyncFn<'a, Arg: 'a, Res: 'a>: 'a {
    fn wrap(self) -> BoxAsyncFn<'a, Arg, Res>;
}

impl<'a, F: 'a, Arg: 'a, Res: 'a, Fut: 'a> AsyncFn<'a, Arg, Res> for F
where
    F: FnOnce(Arg) -> Fut,
    Fut: Send + Future<Output = Res>,
{
    fn wrap(self) -> BoxAsyncFn<'a, Arg, Res> {
print_file_path_and_function_name!();

        Box::new(move |a: Arg| self(a).boxed())
    }
}
