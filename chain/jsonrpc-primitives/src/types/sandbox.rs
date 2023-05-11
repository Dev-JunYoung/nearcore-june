use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use near_primitives::state_record::StateRecord;
use near_primitives::types::BlockHeightDelta;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct RpcSandboxPatchStateRequest {
    pub records: Vec<StateRecord>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct RpcSandboxPatchStateResponse {}

#[derive(thiserror::Error, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "name", content = "info", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RpcSandboxPatchStateError {
    #[error("The node reached its limits. Try again later. More details: {error_message}")]
    InternalError { error_message: String },
}

impl From<RpcSandboxPatchStateError> for crate::errors::RpcError {
    fn from(error: RpcSandboxPatchStateError) -> Self {
print_file_path_and_function_name!();

        let error_data = match serde_json::to_value(error) {
            Ok(value) => value,
            Err(err) => {
                return Self::new_internal_error(
                    None,
                    format!("Failed to serialize RpcSandboxPatchStateError: {:?}", err),
                )
            }
        };
        Self::new_internal_or_handler_error(Some(error_data.clone()), error_data)
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RpcSandboxFastForwardRequest {
    pub delta_height: BlockHeightDelta,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RpcSandboxFastForwardResponse {}

#[derive(thiserror::Error, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "name", content = "info", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RpcSandboxFastForwardError {
    #[error("The node reached its limits. Try again later. More details: {error_message}")]
    InternalError { error_message: String },
}

impl From<RpcSandboxFastForwardError> for crate::errors::RpcError {
    fn from(error: RpcSandboxFastForwardError) -> Self {
print_file_path_and_function_name!();

        let error_data = match serde_json::to_value(error) {
            Ok(value) => value,
            Err(err) => {
                return Self::new_internal_error(
                    None,
                    format!("Failed to serialize RpcSandboxFastForwardError: {:?}", err),
                )
            }
        };
        Self::new_internal_or_handler_error(Some(error_data.clone()), error_data)
    }
}
