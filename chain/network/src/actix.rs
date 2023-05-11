use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use anyhow::anyhow;

// A system thread which is joined on drop.
// TODO: replace with std::thread::ScopedJoinHandle once it is stable.
pub struct Thread(Option<std::thread::JoinHandle<anyhow::Result<()>>>);

impl Thread {
    pub fn spawn<F: Send + 'static + FnOnce() -> anyhow::Result<()>>(f: F) -> Self {
print_file_path_and_function_name!();

        Self(Some(std::thread::spawn(f)))
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
print_file_path_and_function_name!();

        let res = self.0.take().unwrap().join();
        // Panic, unless we are in test and are already panicking.
        // A double panic prevents "cargo test" from displaying error message.
        if !std::thread::panicking() {
            res.unwrap().unwrap();
        }
    }
}

pub struct ActixSystem<A: actix::Actor> {
    pub addr: actix::Addr<A>,
    system: actix::System,
    // dropping _thread has a side effect of joining the system thread.
    // Still, linter considers it a dead_code, so "_" is needed to silence it.
    _thread: Thread,
}

impl<A: actix::Actor> ActixSystem<A> {
    pub async fn spawn<F: Send + 'static + FnOnce() -> actix::Addr<A>>(f: F) -> Self {
print_file_path_and_function_name!();

        let (send, recv) = tokio::sync::oneshot::channel();
        let thread = Thread::spawn(move || {
            let s = actix::System::new();
            s.block_on(async move {
                let system = actix::System::current();
                let addr = f();
                send.send((system, addr)).map_err(|_| anyhow!("send failed"))
            })
            .unwrap();
            s.run().unwrap();
            Ok(())
        });
        let (system, addr) = recv.await.unwrap();
        Self { addr, system, _thread: thread }
    }
}

impl<A: actix::Actor> Drop for ActixSystem<A> {
    fn drop(&mut self) {
print_file_path_and_function_name!();

        self.system.stop();
    }
}
