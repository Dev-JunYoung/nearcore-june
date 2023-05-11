use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use serde_json::Value;

use near_client_primitives::types::GetBlockError;
use near_jsonrpc_primitives::errors::RpcParseError;
use near_jsonrpc_primitives::types::blocks::{RpcBlockError, RpcBlockRequest};
use near_primitives::types::BlockReference;

use super::{Params, RpcFrom, RpcRequest};

impl RpcRequest for RpcBlockRequest {
    fn parse(value: Value) -> Result<Self, RpcParseError> {
print_file_path_and_function_name!();

        let block_reference = Params::new(value)
            .try_singleton(|block_id| Ok(BlockReference::BlockId(block_id)))
            .unwrap_or_parse()?;
        Ok(Self { block_reference })
    }
}

impl RpcFrom<actix::MailboxError> for RpcBlockError {
    fn rpc_from(error: actix::MailboxError) -> Self {
print_file_path_and_function_name!();

        Self::InternalError { error_message: error.to_string() }
    }
}

impl RpcFrom<GetBlockError> for RpcBlockError {
    fn rpc_from(error: GetBlockError) -> Self {
print_file_path_and_function_name!();

        match error {
            GetBlockError::UnknownBlock { error_message } => Self::UnknownBlock { error_message },
            GetBlockError::NotSyncedYet => Self::NotSyncedYet,
            GetBlockError::IOError { error_message } => Self::InternalError { error_message },
            GetBlockError::Unreachable { ref error_message } => {
                tracing::warn!(target: "jsonrpc", "Unreachable error occurred: {}", error_message);
                crate::metrics::RPC_UNREACHABLE_ERROR_COUNT
                    .with_label_values(&["RpcBlockError"])
                    .inc();
                Self::InternalError { error_message: error.to_string() }
            }
        }
    }
}
