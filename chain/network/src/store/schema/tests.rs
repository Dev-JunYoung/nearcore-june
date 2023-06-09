use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use super::*;
use crate::network_protocol::testonly as data;
use crate::testonly::make_rng;

#[test]
fn borsh_wrapper_is_transparent() {
    let mut rng = make_rng(423423);
    let rng = &mut rng;
    let s1 = data::make_secret_key(rng);
    let s2 = data::make_secret_key(rng);
    let e = data::make_edge(&s1, &s2, 1);
    assert_eq!(Borsh(e.clone()).try_to_vec().unwrap(), e.try_to_vec().unwrap());
}
