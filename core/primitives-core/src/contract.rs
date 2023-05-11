use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use crate::hash::{hash as sha256, CryptoHash};

pub struct ContractCode {
    code: Vec<u8>,
    hash: CryptoHash,
}

impl ContractCode {
    pub fn new(code: Vec<u8>, hash: Option<CryptoHash>) -> ContractCode {
print_file_path_and_function_name!();

        let hash = hash.unwrap_or_else(|| sha256(&code));
        debug_assert_eq!(hash, sha256(&code));

        ContractCode { code, hash }
    }

    pub fn code(&self) -> &[u8] {
print_file_path_and_function_name!();

        self.code.as_slice()
    }

    pub fn into_code(self) -> Vec<u8> {
print_file_path_and_function_name!();

        self.code
    }

    pub fn hash(&self) -> &CryptoHash {
print_file_path_and_function_name!();

        &self.hash
    }
}
