// use blockbuster::Utc;
// use blockbuster::DepthGuard;
// use blockbuster::DEPTH_COUNTER;
// use blockbuster::TOTAL_COUNTER;
// use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use near_vm_errors::{FunctionCallError, VMRunnerError};

pub trait IntoVMError {
    fn into_vm_error(self) -> Result<FunctionCallError, VMRunnerError>;
}

#[derive(Debug, PartialEq)]
pub enum ContractPrecompilatonResult {
    ContractCompiled,
    ContractAlreadyInCache,
    CacheNotAvailable,
}
