use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use crate::stun;
use near_async::time;
use near_o11y::testonly::init_test_logger;

#[tokio::test]
async fn test_query() {
print_file_path_and_function_name!();

    init_test_logger();
    let clock = time::FakeClock::default();
    let server = stun::testonly::Server::new().await;
    let ip = stun::query(&clock.clock(), &server.addr()).await.unwrap();
    assert_eq!(std::net::Ipv6Addr::LOCALHOST, ip);
    server.close().await;
}
