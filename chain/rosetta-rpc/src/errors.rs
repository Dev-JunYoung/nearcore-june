use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


#[derive(Debug, strum::EnumIter, thiserror::Error)]
pub(crate) enum ErrorKind {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Wrong network: {0}")]
    WrongNetwork(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("Internal invariant violation: {0}")]
    InternalInvariantError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

pub(crate) type Result<T> = std::result::Result<T, ErrorKind>;

impl From<actix::MailboxError> for ErrorKind {
    fn from(err: actix::MailboxError) -> Self {
print_file_path_and_function_name!();

        Self::InternalError(format!(
            "Server seems to be under a heavy load thus reaching a limit of Actix queue: {}",
            err
        ))
    }
}

impl From<tokio::time::error::Elapsed> for ErrorKind {
    fn from(_: tokio::time::error::Elapsed) -> Self {
print_file_path_and_function_name!();

        Self::Timeout("The operation timed out.".to_string())
    }
}

impl From<near_client::TxStatusError> for ErrorKind {
    fn from(err: near_client::TxStatusError) -> Self {
print_file_path_and_function_name!();

        match err {
            near_client::TxStatusError::ChainError(err) => Self::InternalInvariantError(format!(
                "Transaction could not be found due to an internal error: {:?}",
                err
            )),
            near_client::TxStatusError::MissingTransaction(err) => {
                Self::NotFound(format!("Transaction is missing: {:?}", err))
            }
            near_client::TxStatusError::InternalError(_)
            | near_client::TxStatusError::TimeoutError => {
                // TODO: remove the statuses from TxStatusError since they are
                // never constructed by the view client (it is a leak of
                // abstraction introduced in JSONRPC)
                Self::InternalInvariantError(format!(
                    "TxStatusError reached unexpected state: {:?}",
                    err
                ))
            }
        }
    }
}

impl From<near_client_primitives::types::GetStateChangesError> for ErrorKind {
    fn from(err: near_client_primitives::types::GetStateChangesError) -> Self {
print_file_path_and_function_name!();

        match err {
            near_client_primitives::types::GetStateChangesError::IOError { error_message } => {
                Self::InternalError(error_message)
            }
            near_client_primitives::types::GetStateChangesError::NotSyncedYet => {
                Self::NotFound(err.to_string())
            }
            near_client_primitives::types::GetStateChangesError::UnknownBlock { error_message } => {
                Self::NotFound(error_message)
            }
            near_client_primitives::types::GetStateChangesError::Unreachable { error_message } => {
                Self::InternalError(error_message)
            }
        }
    }
}
