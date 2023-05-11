use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use std::collections::HashMap;
use std::sync::Arc;

use near_crypto::{EmptySigner, InMemorySigner, KeyType, PublicKey, SecretKey, Signature, Signer};
use near_primitives_core::types::ProtocolVersion;

use crate::account::{AccessKey, AccessKeyPermission, Account};
use crate::block::Block;
use crate::block_header::{BlockHeader, BlockHeaderV3};
use crate::errors::EpochError;
use crate::hash::CryptoHash;
use crate::merkle::PartialMerkleTree;
use crate::num_rational::Ratio;
use crate::sharding::ShardChunkHeader;
use crate::transaction::{
    Action, AddKeyAction, CreateAccountAction, DeleteAccountAction, DeleteKeyAction,
    DeployContractAction, FunctionCallAction, SignedTransaction, StakeAction, Transaction,
    TransferAction,
};
use crate::types::{AccountId, Balance, EpochId, EpochInfoProvider, Gas, Nonce};
use crate::validator_signer::{InMemoryValidatorSigner, ValidatorSigner};
use crate::version::PROTOCOL_VERSION;
use crate::views::{ExecutionStatusView, FinalExecutionOutcomeView, FinalExecutionStatus};

pub fn account_new(amount: Balance, code_hash: CryptoHash) -> Account {
print_file_path_and_function_name!();

    Account::new(amount, 0, code_hash, std::mem::size_of::<Account>() as u64)
}

impl Transaction {
    pub fn new(
        signer_id: AccountId,
        public_key: PublicKey,
        receiver_id: AccountId,
        nonce: Nonce,
        block_hash: CryptoHash,
    ) -> Self {
print_file_path_and_function_name!();

        Self { signer_id, public_key, nonce, receiver_id, block_hash, actions: vec![] }
    }

    pub fn sign(self, signer: &dyn Signer) -> SignedTransaction {
print_file_path_and_function_name!();

        let signature = signer.sign(self.get_hash_and_size().0.as_ref());
        SignedTransaction::new(signature, self)
    }

    pub fn create_account(mut self) -> Self {
print_file_path_and_function_name!();

        self.actions.push(Action::CreateAccount(CreateAccountAction {}));
        self
    }

    pub fn deploy_contract(mut self, code: Vec<u8>) -> Self {
print_file_path_and_function_name!();

        self.actions.push(Action::DeployContract(DeployContractAction { code }));
        self
    }

    pub fn function_call(
        mut self,
        method_name: String,
        args: Vec<u8>,
        gas: Gas,
        deposit: Balance,
    ) -> Self {
print_file_path_and_function_name!();

        self.actions.push(Action::FunctionCall(FunctionCallAction {
            method_name,
            args,
            gas,
            deposit,
        }));
        self
    }

    pub fn transfer(mut self, deposit: Balance) -> Self {
print_file_path_and_function_name!();

        self.actions.push(Action::Transfer(TransferAction { deposit }));
        self
    }

    pub fn stake(mut self, stake: Balance, public_key: PublicKey) -> Self {
print_file_path_and_function_name!();

        self.actions.push(Action::Stake(StakeAction { stake, public_key }));
        self
    }
    pub fn add_key(mut self, public_key: PublicKey, access_key: AccessKey) -> Self {
print_file_path_and_function_name!();

        self.actions.push(Action::AddKey(AddKeyAction { public_key, access_key }));
        self
    }

    pub fn delete_key(mut self, public_key: PublicKey) -> Self {
print_file_path_and_function_name!();

        self.actions.push(Action::DeleteKey(DeleteKeyAction { public_key }));
        self
    }

    pub fn delete_account(mut self, beneficiary_id: AccountId) -> Self {
print_file_path_and_function_name!();

        self.actions.push(Action::DeleteAccount(DeleteAccountAction { beneficiary_id }));
        self
    }
}

impl SignedTransaction {
    pub fn from_actions(
        nonce: Nonce,
        signer_id: AccountId,
        receiver_id: AccountId,
        signer: &dyn Signer,
        actions: Vec<Action>,
        block_hash: CryptoHash,
    ) -> Self {
print_file_path_and_function_name!();

        Transaction {
            nonce,
            signer_id,
            public_key: signer.public_key(),
            receiver_id,
            block_hash,
            actions,
        }
        .sign(signer)
    }

    pub fn send_money(
        nonce: Nonce,
        signer_id: AccountId,
        receiver_id: AccountId,
        signer: &dyn Signer,
        deposit: Balance,
        block_hash: CryptoHash,
    ) -> Self {
print_file_path_and_function_name!();

        Self::from_actions(
            nonce,
            signer_id,
            receiver_id,
            signer,
            vec![Action::Transfer(TransferAction { deposit })],
            block_hash,
        )
    }

    pub fn stake(
        nonce: Nonce,
        signer_id: AccountId,
        signer: &dyn Signer,
        stake: Balance,
        public_key: PublicKey,
        block_hash: CryptoHash,
    ) -> Self {
print_file_path_and_function_name!();

        Self::from_actions(
            nonce,
            signer_id.clone(),
            signer_id,
            signer,
            vec![Action::Stake(StakeAction { stake, public_key })],
            block_hash,
        )
    }

    pub fn create_account(
        nonce: Nonce,
        originator: AccountId,
        new_account_id: AccountId,
        amount: Balance,
        public_key: PublicKey,
        signer: &dyn Signer,
        block_hash: CryptoHash,
    ) -> Self {
print_file_path_and_function_name!();

        Self::from_actions(
            nonce,
            originator,
            new_account_id,
            signer,
            vec![
                Action::CreateAccount(CreateAccountAction {}),
                Action::AddKey(AddKeyAction {
                    public_key,
                    access_key: AccessKey { nonce: 0, permission: AccessKeyPermission::FullAccess },
                }),
                Action::Transfer(TransferAction { deposit: amount }),
            ],
            block_hash,
        )
    }

    pub fn create_contract(
        nonce: Nonce,
        originator: AccountId,
        new_account_id: AccountId,
        code: Vec<u8>,
        amount: Balance,
        public_key: PublicKey,
        signer: &dyn Signer,
        block_hash: CryptoHash,
    ) -> Self {
print_file_path_and_function_name!();

        Self::from_actions(
            nonce,
            originator,
            new_account_id,
            signer,
            vec![
                Action::CreateAccount(CreateAccountAction {}),
                Action::AddKey(AddKeyAction {
                    public_key,
                    access_key: AccessKey { nonce: 0, permission: AccessKeyPermission::FullAccess },
                }),
                Action::Transfer(TransferAction { deposit: amount }),
                Action::DeployContract(DeployContractAction { code }),
            ],
            block_hash,
        )
    }

    pub fn call(
        nonce: Nonce,
        signer_id: AccountId,
        receiver_id: AccountId,
        signer: &dyn Signer,
        deposit: Balance,
        method_name: String,
        args: Vec<u8>,
        gas: Gas,
        block_hash: CryptoHash,
    ) -> Self {
print_file_path_and_function_name!();

        Self::from_actions(
            nonce,
            signer_id,
            receiver_id,
            signer,
            vec![Action::FunctionCall(FunctionCallAction { args, method_name, gas, deposit })],
            block_hash,
        )
    }

    pub fn delete_account(
        nonce: Nonce,
        signer_id: AccountId,
        receiver_id: AccountId,
        beneficiary_id: AccountId,
        signer: &dyn Signer,
        block_hash: CryptoHash,
    ) -> Self {
print_file_path_and_function_name!();

        Self::from_actions(
            nonce,
            signer_id,
            receiver_id,
            signer,
            vec![Action::DeleteAccount(DeleteAccountAction { beneficiary_id })],
            block_hash,
        )
    }

    pub fn empty(block_hash: CryptoHash) -> Self {
print_file_path_and_function_name!();

        Self::from_actions(
            0,
            "test".parse().unwrap(),
            "test".parse().unwrap(),
            &EmptySigner {},
            vec![],
            block_hash,
        )
    }
}

impl BlockHeader {
    pub fn get_mut(&mut self) -> &mut BlockHeaderV3 {
print_file_path_and_function_name!();

        match self {
            BlockHeader::BlockHeaderV1(_) | BlockHeader::BlockHeaderV2(_) => {
                panic!("old header should not appear in tests")
            }
            BlockHeader::BlockHeaderV3(header) => Arc::make_mut(header),
        }
    }

    pub fn set_latest_protocol_version(&mut self, latest_protocol_version: ProtocolVersion) {
print_file_path_and_function_name!();

        match self {
            BlockHeader::BlockHeaderV1(header) => {
                let header = Arc::make_mut(header);
                header.inner_rest.latest_protocol_version = latest_protocol_version;
            }
            BlockHeader::BlockHeaderV2(header) => {
                let header = Arc::make_mut(header);
                header.inner_rest.latest_protocol_version = latest_protocol_version;
            }
            BlockHeader::BlockHeaderV3(header) => {
                let header = Arc::make_mut(header);
                header.inner_rest.latest_protocol_version = latest_protocol_version;
            }
        }
    }

    pub fn resign(&mut self, signer: &dyn ValidatorSigner) {
print_file_path_and_function_name!();

        let (hash, signature) = signer.sign_block_header_parts(
            *self.prev_hash(),
            &self.inner_lite_bytes(),
            &self.inner_rest_bytes(),
        );
        match self {
            BlockHeader::BlockHeaderV1(header) => {
                let header = Arc::make_mut(header);
                header.hash = hash;
                header.signature = signature;
            }
            BlockHeader::BlockHeaderV2(header) => {
                let header = Arc::make_mut(header);
                header.hash = hash;
                header.signature = signature;
            }
            BlockHeader::BlockHeaderV3(header) => {
                let header = Arc::make_mut(header);
                header.hash = hash;
                header.signature = signature;
            }
        }
    }
}

/// Builder class for blocks to make testing easier.
/// # Examples
///
/// // TODO(mm-near): change it to doc-tested code once we have easy way to create a genesis block.
/// let signer = EmptyValidatorSigner::default();
/// let test_block = test_utils::TestBlockBuilder::new(prev, signer).height(33).build();
///

pub struct TestBlockBuilder {
    prev: Block,
    signer: Arc<dyn ValidatorSigner>,
    height: u64,
    epoch_id: EpochId,
    next_epoch_id: EpochId,
    next_bp_hash: CryptoHash,
    approvals: Vec<Option<Signature>>,
    block_merkle_root: CryptoHash,
}

impl TestBlockBuilder {
    pub fn new(prev: &Block, signer: Arc<dyn ValidatorSigner>) -> Self {
print_file_path_and_function_name!();

        let mut tree = PartialMerkleTree::default();
        tree.insert(*prev.hash());

        Self {
            prev: prev.clone(),
            signer: signer.clone(),
            height: prev.header().height() + 1,
            epoch_id: prev.header().epoch_id().clone(),
            next_epoch_id: if prev.header().prev_hash() == &CryptoHash::default() {
                EpochId(*prev.hash())
            } else {
                prev.header().next_epoch_id().clone()
            },
            next_bp_hash: *prev.header().next_bp_hash(),
            approvals: vec![],
            block_merkle_root: tree.root(),
        }
    }
    pub fn height(mut self, height: u64) -> Self {
print_file_path_and_function_name!();

        self.height = height;
        self
    }
    pub fn epoch_id(mut self, epoch_id: EpochId) -> Self {
print_file_path_and_function_name!();

        self.epoch_id = epoch_id;
        self
    }
    pub fn next_epoch_id(mut self, next_epoch_id: EpochId) -> Self {
print_file_path_and_function_name!();

        self.next_epoch_id = next_epoch_id;
        self
    }
    pub fn next_bp_hash(mut self, next_bp_hash: CryptoHash) -> Self {
print_file_path_and_function_name!();

        self.next_bp_hash = next_bp_hash;
        self
    }
    pub fn approvals(mut self, approvals: Vec<Option<Signature>>) -> Self {
print_file_path_and_function_name!();

        self.approvals = approvals;
        self
    }

    /// Updates the merkle tree by adding the previous hash, and updates the new block's merkle_root.
    pub fn block_merkle_tree(mut self, block_merkle_tree: &mut PartialMerkleTree) -> Self {
print_file_path_and_function_name!();

        block_merkle_tree.insert(*self.prev.hash());
        self.block_merkle_root = block_merkle_tree.root();
        self
    }

    pub fn build(self) -> Block {
print_file_path_and_function_name!();

        Block::produce(
            PROTOCOL_VERSION,
            PROTOCOL_VERSION,
            self.prev.header(),
            self.height,
            self.prev.header().block_ordinal() + 1,
            self.prev.chunks().iter().cloned().collect(),
            self.epoch_id,
            self.next_epoch_id,
            None,
            self.approvals,
            Ratio::new(0, 1),
            0,
            0,
            Some(0),
            vec![],
            vec![],
            self.signer.as_ref(),
            self.next_bp_hash,
            self.block_merkle_root,
            None,
        )
    }
}

impl Block {
    pub fn mut_header(&mut self) -> &mut BlockHeader {
print_file_path_and_function_name!();

        match self {
            Block::BlockV1(block) => {
                let block = Arc::make_mut(block);
                &mut block.header
            }
            Block::BlockV2(block) => {
                let block = Arc::make_mut(block);
                &mut block.header
            }
        }
    }

    pub fn set_chunks(&mut self, chunks: Vec<ShardChunkHeader>) {
print_file_path_and_function_name!();

        match self {
            Block::BlockV1(block) => {
                let block = Arc::make_mut(block);
                let legacy_chunks = chunks
                    .into_iter()
                    .map(|chunk| match chunk {
                        ShardChunkHeader::V1(header) => header,
                        ShardChunkHeader::V2(_) => {
                            panic!("Attempted to set V1 block chunks with V2")
                        }
                        ShardChunkHeader::V3(_) => {
                            panic!("Attempted to set V1 block chunks with V3")
                        }
                    })
                    .collect();
                block.chunks = legacy_chunks;
            }
            Block::BlockV2(block) => {
                let block = Arc::make_mut(block);
                block.chunks = chunks;
            }
        }
    }
}

#[derive(Default)]
pub struct MockEpochInfoProvider {
    pub validators: HashMap<AccountId, Balance>,
}

impl MockEpochInfoProvider {
    pub fn new(validators: impl Iterator<Item = (AccountId, Balance)>) -> Self {
print_file_path_and_function_name!();

        MockEpochInfoProvider { validators: validators.collect() }
    }
}

impl EpochInfoProvider for MockEpochInfoProvider {
    fn validator_stake(
        &self,
        _epoch_id: &EpochId,
        _last_block_hash: &CryptoHash,
        account_id: &AccountId,
    ) -> Result<Option<Balance>, EpochError> {
print_file_path_and_function_name!();

        Ok(self.validators.get(account_id).cloned())
    }

    fn validator_total_stake(
        &self,
        _epoch_id: &EpochId,
        _last_block_hash: &CryptoHash,
    ) -> Result<Balance, EpochError> {
print_file_path_and_function_name!();

        Ok(self.validators.values().sum())
    }

    fn minimum_stake(&self, _prev_block_hash: &CryptoHash) -> Result<Balance, EpochError> {
print_file_path_and_function_name!();

        Ok(0)
    }
}

/// Encode array of `u64` to be passed as a smart contract argument.
pub fn encode(xs: &[u64]) -> Vec<u8> {
print_file_path_and_function_name!();

    xs.iter().flat_map(|it| it.to_le_bytes()).collect()
}

// Helper function that creates a new signer for a given account, that uses the account name as seed.
// Should be used only in tests.
pub fn create_test_signer(account_name: &str) -> InMemoryValidatorSigner {
print_file_path_and_function_name!();

    InMemoryValidatorSigner::from_seed(
        account_name.parse().unwrap(),
        KeyType::ED25519,
        account_name,
    )
}

/// Helper function that creates a new signer for a given account, that uses the account name as seed.
///
/// This also works for predefined implicit accounts, where the signer will use the implicit key.
///
/// Should be used only in tests.
pub fn create_user_test_signer(account_name: &str) -> InMemorySigner {
print_file_path_and_function_name!();

    let account_id = account_name.parse().unwrap();
    if account_id == implicit_test_account() {
        InMemorySigner::from_secret_key(account_id, implicit_test_account_secret())
    } else {
        InMemorySigner::from_seed(account_id, KeyType::ED25519, account_name)
    }
}

/// A fixed implicit account for which tests can know the private key.
pub fn implicit_test_account() -> AccountId {
print_file_path_and_function_name!();

    "061b1dd17603213b00e1a1e53ba060ad427cef4887bd34a5e0ef09010af23b0a".parse().unwrap()
}

/// Private key for the fixed implicit test account.
pub fn implicit_test_account_secret() -> SecretKey {
print_file_path_and_function_name!();

    "ed25519:5roj6k68kvZu3UEJFyXSfjdKGrodgZUfFLZFpzYXWtESNsLWhYrq3JGi4YpqeVKuw1m9R2TEHjfgWT1fjUqB1DNy".parse().unwrap()
}

impl FinalExecutionOutcomeView {
    #[track_caller]
    /// Check transaction and all transitive receipts for success status.
    pub fn assert_success(&self) {
print_file_path_and_function_name!();

        assert!(matches!(self.status, FinalExecutionStatus::SuccessValue(_)));
        for (i, receipt) in self.receipts_outcome.iter().enumerate() {
            assert!(
                matches!(
                    receipt.outcome.status,
                    ExecutionStatusView::SuccessReceiptId(_) | ExecutionStatusView::SuccessValue(_),
                ),
                "receipt #{i} failed: {receipt:?}",
            );
        }
    }
}
