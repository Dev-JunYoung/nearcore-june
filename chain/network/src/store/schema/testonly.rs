use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use crate::store::schema::{Column, Error, Format};

impl super::Store {
    pub fn iter<C: Column>(
        &self,
    ) -> impl Iterator<Item = Result<(<C::Key as Format>::T, <C::Value as Format>::T), Error>> + '_
    {
print_file_path_and_function_name!();

        debug_assert!(!C::COL.is_rc());
        self.0
            .iter_raw_bytes(C::COL)
            .map(|item| item.and_then(|(k, v)| Ok((C::Key::decode(&k)?, C::Value::decode(&v)?))))
    }
}
