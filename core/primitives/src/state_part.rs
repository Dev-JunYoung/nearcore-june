use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


// to specify a part we always specify both part_id and num_parts together
#[derive(Copy, Clone, Debug)]
pub struct PartId {
    pub idx: u64,
    pub total: u64,
}
impl PartId {
    pub fn new(part_id: u64, num_parts: u64) -> PartId {
print_file_path_and_function_name!();

        assert!(part_id < num_parts);
        PartId { idx: part_id, total: num_parts }
    }
}
