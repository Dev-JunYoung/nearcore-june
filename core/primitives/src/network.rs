use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use crate::hash::CryptoHash;
use crate::types::{AccountId, EpochId};
use borsh::{BorshDeserialize, BorshSerialize};
use near_crypto::{KeyType, PublicKey, SecretKey, Signature};
use std::fmt;
use std::hash::Hash;
use std::sync::Arc;

/// Peer id is the public key.
#[derive(
    BorshSerialize,
    BorshDeserialize,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct PeerId(Arc<PublicKey>);

impl PeerId {
    pub fn new(key: PublicKey) -> Self {
print_file_path_and_function_name!();

        Self(Arc::new(key))
    }

    pub fn public_key(&self) -> &PublicKey {
print_file_path_and_function_name!();

        &self.0
    }
}

impl PeerId {
    pub fn random() -> Self {
print_file_path_and_function_name!();

        PeerId::new(SecretKey::from_random(KeyType::ED25519).public_key())
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
print_file_path_and_function_name!();

        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
print_file_path_and_function_name!();

        write!(f, "{}", self.0)
    }
}

/// Account announcement information
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Clone, Debug, Hash)]
pub struct AnnounceAccount {
    /// AccountId to be announced.
    pub account_id: AccountId,
    /// PeerId from the owner of the account.
    pub peer_id: PeerId,
    /// This announcement is only valid for this `epoch`.
    pub epoch_id: EpochId,
    /// Signature using AccountId associated secret key.
    pub signature: Signature,
}

impl AnnounceAccount {
    /// We hash only (account_id, peer_id, epoch_id). There is no need hash the signature
    /// as it's uniquely determined the the triple.
    pub fn build_header_hash(
        account_id: &AccountId,
        peer_id: &PeerId,
        epoch_id: &EpochId,
    ) -> CryptoHash {
print_file_path_and_function_name!();

        CryptoHash::hash_borsh((account_id, peer_id, epoch_id))
    }

    pub fn hash(&self) -> CryptoHash {
print_file_path_and_function_name!();

        AnnounceAccount::build_header_hash(&self.account_id, &self.peer_id, &self.epoch_id)
    }
}
