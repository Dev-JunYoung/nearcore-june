use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


#[macro_use]
extern crate bencher;

use bencher::Bencher;
use rand::random;

use near_primitives::shard_layout::ShardUId;
use near_store::test_utils::create_tries;
use near_store::Trie;

fn rand_bytes() -> Vec<u8> {
print_file_path_and_function_name!();

    (0..10).map(|_| random::<u8>()).collect()
}

fn trie_lookup(bench: &mut Bencher) {
print_file_path_and_function_name!();

    let (changed_keys, trie) = {
        let tries = create_tries();

        let trie = tries.get_trie_for_shard(ShardUId::single_shard(), Trie::EMPTY_ROOT);
        let mut changes = vec![];
        for _ in 0..100 {
            changes.push((rand_bytes(), Some(rand_bytes())));
        }
        let changed_keys =
            changes.iter().map(|(key, _value)| key.clone()).collect::<Vec<Vec<u8>>>();
        let trie_changes = trie.update(changes).unwrap();
        let mut state_update = tries.store_update();
        let root = tries.apply_all(&trie_changes, ShardUId::single_shard(), &mut state_update);
        state_update.commit().expect("Failed to commit");

        let trie = tries.get_trie_for_shard(ShardUId::single_shard(), root);
        (changed_keys, trie)
    };

    bench.iter(|| {
        for _ in 0..1 {
            for key in changed_keys.iter() {
                trie.get(key).unwrap();
            }
        }
    });
}

fn trie_update(bench: &mut Bencher) {
print_file_path_and_function_name!();

    let tries = create_tries();
    let trie = tries.get_trie_for_shard(ShardUId::single_shard(), Trie::EMPTY_ROOT);
    let mut changes = vec![];
    for _ in 0..100 {
        changes.push((rand_bytes(), Some(rand_bytes())));
    }

    bench.iter(|| {
        let _ = trie.update(changes.iter().cloned());
    });
}

benchmark_group!(benches, trie_lookup, trie_update);
benchmark_main!(benches);
