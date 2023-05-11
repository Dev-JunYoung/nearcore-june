// use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use std::time::Duration;

pub fn measure_performance<F, Message, Result>(
    _class_name: &'static str,
    msg: Message,
    f: F,
) -> Result
where
    F: FnOnce(Message) -> Result,
{
print_file_path_and_function_name!();

    f(msg)
}

pub fn measure_performance_with_debug<F, Message, Result>(
    _class_name: &'static str,
    msg: Message,
    f: F,
) -> Result
where
    F: FnOnce(Message) -> Result,
    for<'a> &'a Message: Into<&'static str>,
{
print_file_path_and_function_name!();

    f(msg)
}

pub fn print_performance_stats(_sleep_time: Duration) {}
