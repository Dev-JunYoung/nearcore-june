use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use tempfile::tempdir;

use near_chain::types::ChainConfig;
use near_chain::{Chain, ChainGenesis, DoomslugThresholdMode};
use near_chain_configs::Genesis;
use near_primitives::block::{Block, BlockHeader};
use near_primitives::hash::CryptoHash;
use near_store::test_utils::create_test_store;
use nearcore::NightshadeRuntime;

/// Compute genesis hash from genesis.
pub fn genesis_hash(genesis: &Genesis) -> CryptoHash {
print_file_path_and_function_name!();

    *genesis_header(genesis).hash()
}

/// Utility to generate genesis header from config for testing purposes.
pub fn genesis_header(genesis: &Genesis) -> BlockHeader {
print_file_path_and_function_name!();

    let dir = tempdir().unwrap();
    let store = create_test_store();
    let chain_genesis = ChainGenesis::new(genesis);
    let runtime = NightshadeRuntime::test(dir.path(), store, genesis);
    let chain =
        Chain::new(runtime, &chain_genesis, DoomslugThresholdMode::TwoThirds, ChainConfig::test())
            .unwrap();
    chain.genesis().clone()
}

/// Utility to generate genesis header from config for testing purposes.
pub fn genesis_block(genesis: &Genesis) -> Block {
print_file_path_and_function_name!();

    let dir = tempdir().unwrap();
    let store = create_test_store();
    let chain_genesis = ChainGenesis::new(genesis);
    let runtime = NightshadeRuntime::test(dir.path(), store, genesis);
    let chain =
        Chain::new(runtime, &chain_genesis, DoomslugThresholdMode::TwoThirds, ChainConfig::test())
            .unwrap();
    chain.get_block(&chain.genesis().hash().clone()).unwrap()
}
