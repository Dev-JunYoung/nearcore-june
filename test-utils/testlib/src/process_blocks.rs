use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use near_chain::{Block, BlockHeader};
use near_primitives::test_utils::create_test_signer;
use std::sync::Arc;

pub fn set_no_chunk_in_block(block: &mut Block, prev_block: &Block) {
print_file_path_and_function_name!();

    let chunk_headers = vec![prev_block.chunks()[0].clone()];
    let mut balance_burnt = 0;
    for chunk in block.chunks().iter() {
        if chunk.height_included() == block.header().height() {
            balance_burnt += chunk.balance_burnt();
        }
    }
    block.set_chunks(chunk_headers.clone());
    match block.mut_header() {
        BlockHeader::BlockHeaderV1(header) => {
            let header = Arc::make_mut(header);
            header.inner_rest.chunk_headers_root =
                Block::compute_chunk_headers_root(&chunk_headers).0;
            header.inner_rest.chunk_tx_root = Block::compute_chunk_tx_root(&chunk_headers);
            header.inner_rest.chunk_receipts_root =
                Block::compute_chunk_receipts_root(&chunk_headers);
            header.inner_lite.prev_state_root = Block::compute_state_root(&chunk_headers);
            header.inner_lite.outcome_root = Block::compute_outcome_root(&chunk_headers);
            header.inner_rest.chunk_mask = vec![false];
            header.inner_rest.gas_price = prev_block.header().gas_price();
            header.inner_rest.total_supply += balance_burnt;
        }
        BlockHeader::BlockHeaderV2(header) => {
            let header = Arc::make_mut(header);
            header.inner_rest.chunk_headers_root =
                Block::compute_chunk_headers_root(&chunk_headers).0;
            header.inner_rest.chunk_tx_root = Block::compute_chunk_tx_root(&chunk_headers);
            header.inner_rest.chunk_receipts_root =
                Block::compute_chunk_receipts_root(&chunk_headers);
            header.inner_lite.prev_state_root = Block::compute_state_root(&chunk_headers);
            header.inner_lite.outcome_root = Block::compute_outcome_root(&chunk_headers);
            header.inner_rest.chunk_mask = vec![false];
            header.inner_rest.gas_price = prev_block.header().gas_price();
            header.inner_rest.total_supply += balance_burnt;
        }
        BlockHeader::BlockHeaderV3(header) => {
            let header = Arc::make_mut(header);
            header.inner_rest.chunk_headers_root =
                Block::compute_chunk_headers_root(&chunk_headers).0;
            header.inner_rest.chunk_tx_root = Block::compute_chunk_tx_root(&chunk_headers);
            header.inner_rest.chunk_receipts_root =
                Block::compute_chunk_receipts_root(&chunk_headers);
            header.inner_lite.prev_state_root = Block::compute_state_root(&chunk_headers);
            header.inner_lite.outcome_root = Block::compute_outcome_root(&chunk_headers);
            header.inner_rest.chunk_mask = vec![false];
            header.inner_rest.gas_price = prev_block.header().gas_price();
            header.inner_rest.total_supply += balance_burnt;
        }
    }
    let validator_signer = create_test_signer("test0");
    block.mut_header().resign(&validator_signer);
}
