use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


pub use near_primitives_core::types::*;

pub type PublicKey = Vec<u8>;
pub type PromiseIndex = u64;
pub type ReceiptIndex = u64;
pub type IteratorIndex = u64;

#[derive(Debug, PartialEq, Clone)]
pub enum ReturnData {
    /// Method returned some value or data.
    Value(Vec<u8>),

    /// The return value of the method should be taken from the return value of another method
    /// identified through receipt index.
    ReceiptIndex(ReceiptIndex),

    /// Method hasn't returned any data or promise.
    None,
}

impl ReturnData {
    /// Function to extract value from ReturnData.
    pub fn as_value(self) -> Option<Vec<u8>> {
print_file_path_and_function_name!();

        match self {
            ReturnData::Value(value) => Some(value),
            _ => None,
        }
    }
}

/// When there is a callback attached to one or more contract calls the execution results of these
/// calls are available to the contract invoked through the callback.
#[derive(Debug, PartialEq)]
pub enum PromiseResult {
    /// Current version of the protocol never returns `PromiseResult::NotReady`.
    NotReady,
    Successful(Vec<u8>),
    Failed,
}
