use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use serde_json::Value;

use near_client_primitives::types::TxStatusError;
use near_jsonrpc_primitives::errors::RpcParseError;
use near_jsonrpc_primitives::types::transactions::{
    RpcBroadcastTransactionRequest, RpcTransactionError, RpcTransactionResponse,
    RpcTransactionStatusCommonRequest, TransactionInfo,
};
use near_primitives::borsh::BorshDeserialize;
use near_primitives::serialize::Base64Bytes;
use near_primitives::transaction::SignedTransaction;
use near_primitives::views::FinalExecutionOutcomeViewEnum;

use super::{Params, RpcFrom, RpcRequest};

impl RpcRequest for RpcBroadcastTransactionRequest {
    fn parse(value: Value) -> Result<Self, RpcParseError> {
print_file_path_and_function_name!();

        let signed_transaction = decode_signed_transaction(value)?;
        Ok(Self { signed_transaction })
    }
}

impl RpcRequest for RpcTransactionStatusCommonRequest {
    fn parse(value: Value) -> Result<Self, RpcParseError> {
print_file_path_and_function_name!();

        let transaction_info = Params::<TransactionInfo>::new(value)
            .try_pair(|hash, account_id| Ok(TransactionInfo::TransactionId { hash, account_id }))
            .unwrap_or_else(|value| {
                decode_signed_transaction(value).map(TransactionInfo::Transaction)
            })?;
        Ok(Self { transaction_info })
    }
}

impl RpcFrom<actix::MailboxError> for RpcTransactionError {
    fn rpc_from(error: actix::MailboxError) -> Self {
print_file_path_and_function_name!();

        Self::InternalError { debug_info: error.to_string() }
    }
}

impl RpcFrom<TxStatusError> for RpcTransactionError {
    fn rpc_from(error: TxStatusError) -> Self {
print_file_path_and_function_name!();

        match error {
            TxStatusError::ChainError(err) => {
                Self::InternalError { debug_info: format!("{:?}", err) }
            }
            TxStatusError::MissingTransaction(requested_transaction_hash) => {
                Self::UnknownTransaction { requested_transaction_hash }
            }
            TxStatusError::InternalError(debug_info) => Self::InternalError { debug_info },
            TxStatusError::TimeoutError => Self::TimeoutError,
        }
    }
}

impl RpcFrom<FinalExecutionOutcomeViewEnum> for RpcTransactionResponse {
    fn rpc_from(final_execution_outcome: FinalExecutionOutcomeViewEnum) -> Self {
print_file_path_and_function_name!();

        Self { final_execution_outcome }
    }
}

fn decode_signed_transaction(value: Value) -> Result<SignedTransaction, RpcParseError> {
print_file_path_and_function_name!();

    let (bytes,) = Params::<(Base64Bytes,)>::parse(value)?;
    SignedTransaction::try_from_slice(&bytes.0)
        .map_err(|err| RpcParseError(format!("Failed to decode transaction: {}", err)))
}