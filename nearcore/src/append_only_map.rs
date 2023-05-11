use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

const POISONED_LOCK_ERR: &str = "The lock was poisoned.";

pub struct AppendOnlyMap<K, V> {
    map: RwLock<HashMap<K, Arc<V>>>,
}

impl<K, V> AppendOnlyMap<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new() -> Self {
print_file_path_and_function_name!();

        Self { map: RwLock::new(HashMap::new()) }
    }

    pub fn get_or_insert<F: FnOnce() -> V>(&self, key: &K, value: F) -> Arc<V> {
print_file_path_and_function_name!();

        let mut map = self.map.write().expect(POISONED_LOCK_ERR);
        if !map.contains_key(key) {
            map.insert(key.clone(), Arc::new(value()));
        }
        map.get(key).unwrap().clone()
    }
}
