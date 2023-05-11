use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use crate::concurrency::ctx;
use crate::concurrency::scope;
use crate::testonly::abort_on_panic;
use near_async::time;

#[tokio::test]
async fn test_run_canceled() {
print_file_path_and_function_name!();

    abort_on_panic();
    ctx::run_canceled(async {
        assert!(ctx::is_canceled());
    })
    .await;
}

#[tokio::test]
async fn test_run_test() {
print_file_path_and_function_name!();

    abort_on_panic();
    let clock = time::FakeClock::default();
    ctx::run_test(clock.clock(), async {
        assert_eq!(ctx::time::now(), clock.now());
        assert_eq!(ctx::time::now_utc(), clock.now_utc());
        clock.advance(time::Duration::seconds(10));
        assert_eq!(ctx::time::now(), clock.now());
    })
    .await;
}

type R<E> = Result<(), E>;

#[tokio::test]
async fn test_run_with_timeout() {
print_file_path_and_function_name!();

    abort_on_panic();
    let sec = near_async::time::Duration::SECOND;
    let clock = time::FakeClock::default();
    let res = ctx::run_test(clock.clock(), async {
        scope::run!(|s| async {
            s.spawn(ctx::run_with_timeout(1000 * sec, async {
                ctx::canceled().await;
                R::Err(9)
            }));
            clock.advance(1001 * sec);
            Ok(())
        })
    })
    .await;
    assert_eq!(Err(9), res);
}

#[tokio::test]
async fn test_sleep_until() {
print_file_path_and_function_name!();

    abort_on_panic();
    let sec = near_async::time::Duration::SECOND;
    let clock = time::FakeClock::default();
    ctx::run_test(clock.clock(), async {
        let _ = scope::run!(|s| async {
            let t = ctx::time::now() + 1000 * sec;
            s.spawn(async move {
                ctx::time::sleep_until(t).await.unwrap();
                R::Err(1)
            });
            clock.advance_until(t + sec);
            Ok(())
        });
        let _ = scope::run!(|s| async {
            let t = ctx::time::now() + 1000 * sec;
            s.spawn(async move {
                assert!(ctx::time::sleep_until(t).await.is_err());
                Ok(())
            });
            clock.advance_until(t - sec);
            R::Err(1)
        });
    })
    .await
}
