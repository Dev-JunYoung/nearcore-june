use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use serde_json::Value;

use near_jsonrpc_primitives::errors::RpcParseError;
use near_jsonrpc_primitives::types::sandbox::{
    RpcSandboxFastForwardError, RpcSandboxFastForwardRequest, RpcSandboxPatchStateError,
    RpcSandboxPatchStateRequest,
};

use super::{Params, RpcFrom, RpcRequest};

impl RpcRequest for RpcSandboxPatchStateRequest {
    fn parse(value: Value) -> Result<Self, RpcParseError> {
print_file_path_and_function_name!();

        Params::parse(value)
    }
}

impl RpcRequest for RpcSandboxFastForwardRequest {
    fn parse(value: Value) -> Result<Self, RpcParseError> {
print_file_path_and_function_name!();

        Params::parse(value)
    }
}

impl RpcFrom<actix::MailboxError> for RpcSandboxPatchStateError {
    fn rpc_from(error: actix::MailboxError) -> Self {
print_file_path_and_function_name!();

        Self::InternalError { error_message: error.to_string() }
    }
}

impl RpcFrom<actix::MailboxError> for RpcSandboxFastForwardError {
    fn rpc_from(error: actix::MailboxError) -> Self {
print_file_path_and_function_name!();

        Self::InternalError { error_message: error.to_string() }
    }
}
