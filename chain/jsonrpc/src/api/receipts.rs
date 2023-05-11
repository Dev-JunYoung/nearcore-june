use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use super::{Params, RpcFrom, RpcRequest};
use near_client_primitives::types::{GetReceipt, GetReceiptError};
use near_jsonrpc_primitives::errors::RpcParseError;
use near_jsonrpc_primitives::types::receipts::{
    ReceiptReference, RpcReceiptError, RpcReceiptRequest,
};
use serde_json::Value;

impl RpcRequest for RpcReceiptRequest {
    fn parse(value: Value) -> Result<Self, RpcParseError> {
print_file_path_and_function_name!();

        Ok(Self { receipt_reference: Params::parse(value)? })
    }
}

impl RpcFrom<actix::MailboxError> for RpcReceiptError {
    fn rpc_from(error: actix::MailboxError) -> Self {
print_file_path_and_function_name!();

        Self::InternalError { error_message: error.to_string() }
    }
}

impl RpcFrom<ReceiptReference> for GetReceipt {
    fn rpc_from(receipt_reference: ReceiptReference) -> Self {
print_file_path_and_function_name!();

        Self { receipt_id: receipt_reference.receipt_id }
    }
}

impl RpcFrom<GetReceiptError> for RpcReceiptError {
    fn rpc_from(error: GetReceiptError) -> Self {
print_file_path_and_function_name!();

        match error {
            GetReceiptError::IOError(error_message) => Self::InternalError { error_message },
            GetReceiptError::UnknownReceipt(hash) => Self::UnknownReceipt { receipt_id: hash },
            GetReceiptError::Unreachable(ref error_message) => {
                tracing::warn!(target: "jsonrpc", "Unreachable error occurred: {}", error_message);
                crate::metrics::RPC_UNREACHABLE_ERROR_COUNT
                    .with_label_values(&["RpcReceiptError"])
                    .inc();
                Self::InternalError { error_message: error.to_string() }
            }
        }
    }
}
