// use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


#[allow(deprecated)]
use std::hash::{Hasher, SipHasher};

/// We not use stable hasher as it could change with Rust releases, so rely on stable SIP hash.
#[allow(deprecated)]
#[derive(Default, Clone)]
pub struct StableHasher(pub SipHasher);

impl StableHasher {
    #[allow(deprecated)]
    pub fn new() -> StableHasher {
print_file_path_and_function_name!();

        StableHasher(SipHasher::new())
    }
}

impl Hasher for StableHasher {
    fn finish(&self) -> u64 {
print_file_path_and_function_name!();

        self.0.finish()
    }
    fn write(&mut self, bytes: &[u8]) {
print_file_path_and_function_name!();

        self.0.write(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::StableHasher;
    use std::hash::Hasher;

    /// Make sure the stable hasher never changes
    #[test]
    fn test_stable_hasher() {
        let mut sh = StableHasher::new();

        sh.write(&[1, 2, 3, 4, 5]);
        let finish = sh.finish();
        assert_eq!(finish, 12661990674860217757)
    }
}
