use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


mod challenges;
mod doomslug;
mod gc;
mod simple_chain;
mod sync_chain;

use crate::block_processing_utils::BlockProcessingArtifact;
use crate::test_utils::process_block_sync;
use crate::{Block, Chain, Error, Provenance};
use near_primitives::account::id::AccountId;
use near_primitives::utils::MaybeValidated;

impl Chain {
    /// A wrapper function around process_block that doesn't trigger all the callbacks
    /// Only used in tests
    pub(crate) fn process_block_test(
        &mut self,
        me: &Option<AccountId>,
        block: Block,
    ) -> Result<(), Error> {
print_file_path_and_function_name!();

        let mut block_processing_artifacts = BlockProcessingArtifact::default();
        process_block_sync(
            self,
            me,
            MaybeValidated::from(block),
            Provenance::PRODUCED,
            &mut block_processing_artifacts,
        )
        .map(|_| {})
    }
}
