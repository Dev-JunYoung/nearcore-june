use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use near_client_primitives::types::GetClientConfigError;
use near_jsonrpc_primitives::types::client_config::RpcClientConfigError;

use super::RpcFrom;

impl RpcFrom<actix::MailboxError> for RpcClientConfigError {
    fn rpc_from(error: actix::MailboxError) -> Self {
print_file_path_and_function_name!();

        Self::InternalError { error_message: error.to_string() }
    }
}

impl RpcFrom<GetClientConfigError> for RpcClientConfigError {
    fn rpc_from(error: GetClientConfigError) -> Self {
print_file_path_and_function_name!();

        match error {
            GetClientConfigError::IOError(error_message) => Self::InternalError { error_message },
            GetClientConfigError::Unreachable(ref error_message) => {
                tracing::warn!(target: "jsonrpc", "Unreachable error occurred: {}", error_message);
                crate::metrics::RPC_UNREACHABLE_ERROR_COUNT
                    .with_label_values(&["RpcClientConfigError"])
                    .inc();
                Self::InternalError { error_message: error.to_string() }
            }
        }
    }
}
