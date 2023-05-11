use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use near_chain::ChainStore;
use near_chain::ChainStoreAccess;
use near_primitives::account::id::AccountId;
use near_primitives::block::Block;
use near_primitives::transaction::SignedTransaction;

/// Returns a list of transactions found in the block.
pub fn dump_tx_from_block(
    chain_store: &ChainStore,
    block: &Block,
    select_account_ids: Option<&[AccountId]>,
) -> Vec<SignedTransaction> {
print_file_path_and_function_name!();

    let chunks = block.chunks();
    let mut res = vec![];
    for (_, chunk_header) in chunks.iter().enumerate() {
        res.extend(
            chain_store
                .get_chunk(&chunk_header.chunk_hash())
                .unwrap()
                .transactions()
                .iter()
                .filter(|signed_transaction| {
                    should_include_signed_transaction(signed_transaction, select_account_ids)
                })
                .cloned()
                .collect::<Vec<_>>(),
        );
    }
    res
}

fn should_include_signed_transaction(
    signed_transaction: &SignedTransaction,
    select_account_ids: Option<&[AccountId]>,
) -> bool {
print_file_path_and_function_name!();

    match select_account_ids {
        None => true,
        Some(specified_ids) => specified_ids.contains(&signed_transaction.transaction.receiver_id),
    }
}
