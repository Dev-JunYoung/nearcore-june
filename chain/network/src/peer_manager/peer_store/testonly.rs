use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use crate::types::KnownPeerState;

impl super::PeerStore {
    pub fn dump(&self) -> Vec<KnownPeerState> {
print_file_path_and_function_name!();

        self.0.lock().peer_states.iter().map(|(_, v)| v.clone()).collect()
    }
}
