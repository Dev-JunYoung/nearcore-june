use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use near_chain_configs::{Genesis, GenesisValidationMode};

#[test]
fn test_load_genesis() {
    Genesis::from_file("res/mainnet_genesis.json", GenesisValidationMode::Full).unwrap();
}
