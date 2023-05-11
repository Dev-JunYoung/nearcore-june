use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use crate::types::ConnectionInfo;

impl super::ConnectionStore {
    pub(crate) fn insert_outbound_connections(&self, outbound: Vec<ConnectionInfo>) {
print_file_path_and_function_name!();

        self.0.update(|mut inner| {
            inner.push_front_outbound(outbound);
            ((), inner)
        });
    }
}
