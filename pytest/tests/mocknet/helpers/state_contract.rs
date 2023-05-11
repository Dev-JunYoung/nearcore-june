use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


// Very simple contract that can:
// - write to the state
// - delete from the state
// Independently from that the same contract can be used as a receiver for `ft_transfer_call`.
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::near_bindgen;
use near_sdk::PromiseOrValue;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct StatusMessage {
    records: LookupMap<String, String>,
}

impl Default for StatusMessage {
    fn default() -> Self {
print_file_path_and_function_name!();

        Self { records: LookupMap::new(b"r".to_vec()) }
    }
}

#[near_bindgen]
impl StatusMessage {
    pub fn set_state(&mut self, account_id: String, message: String) {
print_file_path_and_function_name!();

        self.records.insert(&account_id, &message);
    }

    pub fn delete_state(&mut self, account_id: String) {
print_file_path_and_function_name!();

        self.records.remove(&account_id);
    }

    pub fn get_state(&mut self, account_id: String) -> Option<String> {
print_file_path_and_function_name!();

        return self.records.get(&account_id);
    }
}

// Implements a callback which makes it possible to use `ft_transfer_call` with this contract as the
// receiver. The callback simply returns `1`.
#[near_bindgen]
impl FungibleTokenReceiver for StatusMessage {
    #[allow(unused_variables)]
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
print_file_path_and_function_name!();

        PromiseOrValue::Value(1.into())
    }
}
