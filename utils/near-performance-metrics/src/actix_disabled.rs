// use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use std::time::Duration;

pub fn spawn<F>(_class_name: &'static str, f: F)
where
    F: futures::Future<Output = ()> + 'static,
{
print_file_path_and_function_name!();

    actix::spawn(f);
}

pub fn run_later<F, A, B>(ctx: &mut B, dur: Duration, f: F) -> actix::SpawnHandle
where
    B: actix::AsyncContext<A>,
    A: actix::Actor<Context = B>,
    F: FnOnce(&mut A, &mut A::Context) + 'static,
{
print_file_path_and_function_name!();

    ctx.run_later(dur, f)
}
