use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use std::sync::Arc;

use borsh::BorshDeserialize;
use near_cache::CellLruCache;
use near_chain_primitives::Error;
use near_primitives::sharding::{ChunkHash, PartialEncodedChunk, ShardChunk};
use near_store::{DBCol, Store};

#[cfg(not(feature = "no_cache"))]
const CHUNK_CACHE_SIZE: usize = 1024;
#[cfg(feature = "no_cache")]
const CHUNK_CACHE_SIZE: usize = 1;

pub struct ReadOnlyChunksStore {
    store: Store,
    partial_chunks: CellLruCache<Vec<u8>, Arc<PartialEncodedChunk>>,
    chunks: CellLruCache<Vec<u8>, Arc<ShardChunk>>,
}

impl ReadOnlyChunksStore {
    pub fn new(store: Store) -> Self {
print_file_path_and_function_name!();

        Self {
            store,
            partial_chunks: CellLruCache::new(CHUNK_CACHE_SIZE),
            chunks: CellLruCache::new(CHUNK_CACHE_SIZE),
        }
    }

    fn read_with_cache<'a, T: BorshDeserialize + Clone + 'a>(
        &self,
        col: DBCol,
        cache: &'a CellLruCache<Vec<u8>, T>,
        key: &[u8],
    ) -> std::io::Result<Option<T>> {
print_file_path_and_function_name!();

        if let Some(value) = cache.get(key) {
            return Ok(Some(value));
        }
        if let Some(result) = self.store.get_ser::<T>(col, key)? {
            cache.put(key.to_vec(), result.clone());
            return Ok(Some(result));
        }
        Ok(None)
    }
    pub fn get_partial_chunk(
        &self,
        chunk_hash: &ChunkHash,
    ) -> Result<Arc<PartialEncodedChunk>, Error> {
print_file_path_and_function_name!();

        match self.read_with_cache(DBCol::PartialChunks, &self.partial_chunks, chunk_hash.as_ref())
        {
            Ok(Some(shard_chunk)) => Ok(shard_chunk),
            _ => Err(Error::ChunkMissing(chunk_hash.clone())),
        }
    }
    pub fn get_chunk(&self, chunk_hash: &ChunkHash) -> Result<Arc<ShardChunk>, Error> {
print_file_path_and_function_name!();

        match self.read_with_cache(DBCol::Chunks, &self.chunks, chunk_hash.as_ref()) {
            Ok(Some(shard_chunk)) => Ok(shard_chunk),
            _ => Err(Error::ChunkMissing(chunk_hash.clone())),
        }
    }
}
