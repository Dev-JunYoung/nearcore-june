use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use std::path::PathBuf;
use std::sync::Arc;

use near_actix_test_utils::ShutdownableThread;
use near_chain_configs::Genesis;
use near_crypto::{InMemorySigner, KeyType, Signer};
use near_primitives::types::AccountId;
use nearcore::{start_with_config, NearConfig};

use crate::node::Node;
use crate::user::rpc_user::RpcUser;
use crate::user::User;

pub enum ThreadNodeState {
    Stopped,
    Running(ShutdownableThread),
}

pub struct ThreadNode {
    pub config: NearConfig,
    pub state: ThreadNodeState,
    pub signer: Arc<InMemorySigner>,
    pub dir: tempfile::TempDir,
}

fn start_thread(config: NearConfig, path: PathBuf) -> ShutdownableThread {
print_file_path_and_function_name!();

    ShutdownableThread::start("test", move || {
        start_with_config(&path, config).expect("start_with_config");
    })
}

impl Node for ThreadNode {
    fn genesis(&self) -> &Genesis {
print_file_path_and_function_name!();

        &self.config.genesis
    }

    fn account_id(&self) -> Option<AccountId> {
print_file_path_and_function_name!();

        self.config.validator_signer.as_ref().map(|vs| vs.validator_id().clone())
    }

    fn start(&mut self) {
print_file_path_and_function_name!();

        let handle = start_thread(self.config.clone(), self.dir.path().to_path_buf());
        self.state = ThreadNodeState::Running(handle);
    }

    fn kill(&mut self) {
print_file_path_and_function_name!();

        let state = std::mem::replace(&mut self.state, ThreadNodeState::Stopped);
        match state {
            ThreadNodeState::Stopped => panic!("Node is not running"),
            ThreadNodeState::Running(handle) => {
                handle.shutdown();
                self.state = ThreadNodeState::Stopped;
            }
        }
    }

    fn signer(&self) -> Arc<dyn Signer> {
print_file_path_and_function_name!();

        self.signer.clone()
    }

    fn is_running(&self) -> bool {
print_file_path_and_function_name!();

        match self.state {
            ThreadNodeState::Stopped => false,
            ThreadNodeState::Running(_) => true,
        }
    }

    fn user(&self) -> Box<dyn User> {
print_file_path_and_function_name!();

        let account_id = self.signer.account_id.clone();
        Box::new(RpcUser::new(&self.config.rpc_addr().unwrap(), account_id, self.signer.clone()))
    }

    fn as_thread_ref(&self) -> &ThreadNode {
print_file_path_and_function_name!();

        self
    }

    fn as_thread_mut(&mut self) -> &mut ThreadNode {
print_file_path_and_function_name!();

        self
    }
}

impl ThreadNode {
    /// Side effects: create storage, open database, lock database
    pub fn new(config: NearConfig) -> ThreadNode {
print_file_path_and_function_name!();

        let signer = Arc::new(InMemorySigner::from_seed(
            config.validator_signer.as_ref().unwrap().validator_id().clone(),
            KeyType::ED25519,
            config.validator_signer.as_ref().unwrap().validator_id().as_ref(),
        ));
        ThreadNode {
            config,
            state: ThreadNodeState::Stopped,
            signer,
            dir: tempfile::Builder::new().prefix("thread_node").tempdir().unwrap(),
        }
    }
}
