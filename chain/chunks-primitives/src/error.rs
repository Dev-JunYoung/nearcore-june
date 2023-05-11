use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use std::fmt;

use near_primitives::errors::EpochError;

#[derive(Debug)]
pub enum Error {
    InvalidPartMessage,
    InvalidChunkPartId,
    InvalidChunkShardId,
    InvalidMerkleProof,
    InvalidChunkSignature,
    InvalidChunkHeader,
    InvalidChunk,
    DuplicateChunkHeight,
    UnknownChunk,
    KnownPart,
    ChainError(near_chain_primitives::Error),
    IOError(std::io::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
print_file_path_and_function_name!();

        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
print_file_path_and_function_name!();

        Error::IOError(err)
    }
}

impl From<near_chain_primitives::Error> for Error {
    fn from(err: near_chain_primitives::Error) -> Self {
print_file_path_and_function_name!();

        Error::ChainError(err)
    }
}

impl From<EpochError> for Error {
    fn from(err: EpochError) -> Self {
print_file_path_and_function_name!();

        Error::ChainError(err.into())
    }
}
