use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use std::sync::Arc;
use std::thread;
use std::time::Duration;

use borsh::BorshSerialize;
use futures::{Future, TryFutureExt};

use near_client::StatusResponse;
use near_crypto::{PublicKey, Signer};
use near_jsonrpc::client::{new_client, JsonRpcClient};
use near_jsonrpc_client::ChunkId;
use near_jsonrpc_primitives::errors::ServerError;
use near_jsonrpc_primitives::types::query::{RpcQueryRequest, RpcQueryResponse};
use near_primitives::hash::CryptoHash;
use near_primitives::receipt::Receipt;
use near_primitives::serialize::to_base64;
use near_primitives::transaction::SignedTransaction;
use near_primitives::types::{
    AccountId, BlockHeight, BlockId, BlockReference, MaybeBlockId, ShardId,
};
use near_primitives::views::{
    AccessKeyView, AccountView, BlockView, CallResult, ChunkView, ContractCodeView,
    EpochValidatorInfo, ExecutionOutcomeView, FinalExecutionOutcomeView, QueryRequest,
    ViewStateResult,
};

use crate::user::User;

pub struct RpcUser {
    account_id: AccountId,
    signer: Arc<dyn Signer>,
    addr: String,
}

impl RpcUser {
    fn actix<F, Fut, R>(&self, f: F) -> R
    where
        Fut: Future<Output = R> + 'static,
        F: FnOnce(JsonRpcClient) -> Fut + 'static,
    {
print_file_path_and_function_name!();

        let addr = self.addr.clone();
        actix::System::new()
            .block_on(async move { f(new_client(&format!("http://{}", addr))).await })
    }

    pub fn new(addr: &str, account_id: AccountId, signer: Arc<dyn Signer>) -> RpcUser {
print_file_path_and_function_name!();

        RpcUser { account_id, addr: addr.to_owned(), signer }
    }

    pub fn get_status(&self) -> Option<StatusResponse> {
print_file_path_and_function_name!();

        self.actix(|client| client.status()).ok()
    }

    pub fn query(&self, request: QueryRequest) -> Result<RpcQueryResponse, String> {
print_file_path_and_function_name!();

        let request = RpcQueryRequest { request, block_reference: BlockReference::latest() };
        self.actix(move |client| client.query(request).map_err(|err| err.to_string()))
    }

    pub fn validators(&self, block_id: MaybeBlockId) -> Result<EpochValidatorInfo, String> {
print_file_path_and_function_name!();

        self.actix(move |client| client.validators(block_id).map_err(|err| err.to_string()))
    }
}

impl User for RpcUser {
    fn view_account(&self, account_id: &AccountId) -> Result<AccountView, String> {
print_file_path_and_function_name!();

        let query = QueryRequest::ViewAccount { account_id: account_id.clone() };
        match self.query(query)?.kind {
            near_jsonrpc_primitives::types::query::QueryResponseKind::ViewAccount(account_view) => {
                Ok(account_view)
            }
            _ => Err("Invalid type of response".into()),
        }
    }

    fn view_state(&self, account_id: &AccountId, prefix: &[u8]) -> Result<ViewStateResult, String> {
print_file_path_and_function_name!();

        let query = QueryRequest::ViewState {
            account_id: account_id.clone(),
            prefix: prefix.to_vec().into(),
            include_proof: false,
        };
        match self.query(query)?.kind {
            near_jsonrpc_primitives::types::query::QueryResponseKind::ViewState(
                view_state_result,
            ) => Ok(view_state_result),
            _ => Err("Invalid type of response".into()),
        }
    }

    fn view_contract_code(&self, account_id: &AccountId) -> Result<ContractCodeView, String> {
print_file_path_and_function_name!();

        let query = QueryRequest::ViewCode { account_id: account_id.clone() };
        match self.query(query)?.kind {
            near_jsonrpc_primitives::types::query::QueryResponseKind::ViewCode(
                contract_code_view,
            ) => Ok(contract_code_view),
            _ => Err("Invalid type of response".into()),
        }
    }

    fn view_call(
        &self,
        account_id: &AccountId,
        method_name: &str,
        args: &[u8],
    ) -> Result<CallResult, String> {
print_file_path_and_function_name!();

        let query = QueryRequest::CallFunction {
            account_id: account_id.clone(),
            method_name: method_name.to_string(),
            args: args.to_vec().into(),
        };
        match self.query(query)?.kind {
            near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(call_result) => {
                Ok(call_result)
            }
            _ => Err("Invalid type of response".into()),
        }
    }

    fn add_transaction(&self, transaction: SignedTransaction) -> Result<(), ServerError> {
print_file_path_and_function_name!();

        let bytes = transaction.try_to_vec().unwrap();
        let _ = self.actix(move |client| client.broadcast_tx_async(to_base64(&bytes))).map_err(
            |err| {
                serde_json::from_value::<ServerError>(
                    err.data.expect("server error must carry data"),
                )
                .expect("deserialize server error must be ok")
            },
        )?;
        Ok(())
    }

    fn commit_transaction(
        &self,
        transaction: SignedTransaction,
    ) -> Result<FinalExecutionOutcomeView, ServerError> {
print_file_path_and_function_name!();

        let bytes = transaction.try_to_vec().unwrap();
        let result = self.actix(move |client| client.broadcast_tx_commit(to_base64(&bytes)));
        // Wait for one more block, to make sure all nodes actually apply the state transition.
        let height = self.get_best_height().unwrap();
        while height == self.get_best_height().unwrap() {
            thread::sleep(Duration::from_millis(50));
        }
        match result {
            Ok(outcome) => Ok(outcome),
            Err(err) => Err(serde_json::from_value::<ServerError>(err.data.unwrap()).unwrap()),
        }
    }

    fn add_receipts(
        &self,
        _receipts: Vec<Receipt>,
        _use_flat_storage: bool,
    ) -> Result<(), ServerError> {
print_file_path_and_function_name!();

        // TDDO: figure out if rpc will support this
        unimplemented!()
    }

    fn get_best_height(&self) -> Option<BlockHeight> {
print_file_path_and_function_name!();

        self.get_status().map(|status| status.sync_info.latest_block_height)
    }

    fn get_best_block_hash(&self) -> Option<CryptoHash> {
print_file_path_and_function_name!();

        self.get_status().map(|status| status.sync_info.latest_block_hash)
    }

    fn get_block_by_height(&self, height: BlockHeight) -> Option<BlockView> {
print_file_path_and_function_name!();

        self.actix(move |client| client.block(BlockReference::BlockId(BlockId::Height(height))))
            .ok()
    }

    fn get_block(&self, block_hash: CryptoHash) -> Option<BlockView> {
print_file_path_and_function_name!();

        self.actix(move |client| client.block(BlockReference::BlockId(BlockId::Hash(block_hash))))
            .ok()
    }

    fn get_chunk_by_height(&self, height: BlockHeight, shard_id: ShardId) -> Option<ChunkView> {
print_file_path_and_function_name!();

        self.actix(move |client| {
            client.chunk(ChunkId::BlockShardId(BlockId::Height(height), shard_id))
        })
        .ok()
    }

    fn get_transaction_result(&self, _hash: &CryptoHash) -> ExecutionOutcomeView {
print_file_path_and_function_name!();

        unimplemented!()
    }

    fn get_transaction_final_result(&self, hash: &CryptoHash) -> FinalExecutionOutcomeView {
print_file_path_and_function_name!();

        let account_id = self.account_id.clone();
        let hash = hash.to_string();
        self.actix(move |client| client.tx(hash, account_id)).unwrap()
    }

    fn get_state_root(&self) -> CryptoHash {
print_file_path_and_function_name!();

        self.get_status().map(|status| status.sync_info.latest_state_root).unwrap()
    }

    fn get_access_key(
        &self,
        account_id: &AccountId,
        public_key: &PublicKey,
    ) -> Result<AccessKeyView, String> {
print_file_path_and_function_name!();

        let query = QueryRequest::ViewAccessKey {
            account_id: account_id.clone(),
            public_key: public_key.clone(),
        };
        match self.query(query)?.kind {
            near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(access_key) => {
                Ok(access_key)
            }
            _ => Err("Invalid type of response".into()),
        }
    }

    fn signer(&self) -> Arc<dyn Signer> {
print_file_path_and_function_name!();

        self.signer.clone()
    }

    fn set_signer(&mut self, signer: Arc<dyn Signer>) {
print_file_path_and_function_name!();

        self.signer = signer;
    }
}
