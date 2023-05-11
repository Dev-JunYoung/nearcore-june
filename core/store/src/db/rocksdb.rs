use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use crate::config::Mode;
use crate::db::{refcount, DBIterator, DBOp, DBSlice, DBTransaction, Database, StatsValue};
use crate::{metadata, metrics, DBCol, StoreConfig, StoreStatistics, Temperature};
use ::rocksdb::{
    BlockBasedOptions, Cache, ColumnFamily, Env, IteratorMode, Options, ReadOptions, WriteBatch, DB,
};
use std::io;
use std::path::Path;
use strum::IntoEnumIterator;
use tracing::warn;

mod instance_tracker;
pub(crate) mod snapshot;

/// List of integer RocskDB properties we’re reading when collecting statistics.
///
/// In the end, they are exported as Prometheus metrics.
const CF_PROPERTY_NAMES: [&'static std::ffi::CStr; 1] =
    [::rocksdb::properties::LIVE_SST_FILES_SIZE];

pub struct RocksDB {
    db: DB,
    db_opt: Options,

    /// Map from [`DBCol`] to a column family handler in the RocksDB.
    ///
    /// Rather than accessing this field directly, use [`RocksDB::cf_handle`]
    /// method instead.  It returns `&ColumnFamily` which is what you usually
    /// want.
    cf_handles: enum_map::EnumMap<DBCol, Option<std::ptr::NonNull<ColumnFamily>>>,

    // RAII-style of keeping track of the number of instances of RocksDB and
    // counting total sum of max_open_files.
    _instance_tracker: instance_tracker::InstanceTracker,
}

// DB was already Send+Sync. cf and read_options are const pointers using only functions in
// this file and safe to share across threads.
unsafe impl Send for RocksDB {}
unsafe impl Sync for RocksDB {}

impl RocksDB {
    /// Opens the database.
    ///
    /// `path` specifies location of the database.  It’s assumed that it has
    /// been resolved based on configuration in `store_config` and thus path
    /// configuration in `store_config` is ignored.
    ///
    /// `store_config` specifies other storage configuration such open files
    /// limit or whether to enable collection of statistics.
    ///
    /// `mode` specifies whether to open the database in read/write or read-only
    /// mode.  In the latter case, the database will not be created if it
    /// doesn’t exist nor any migrations will be performed if the database has
    /// database version different than expected.
    ///
    /// `temp` specifies whether the database is cold or hot which affects
    /// whether refcount merge operator is configured on reference counted
    /// column.
    pub fn open(
        path: &Path,
        store_config: &StoreConfig,
        mode: Mode,
        temp: Temperature,
    ) -> io::Result<Self> {
        print_file_path_and_function_name!();
        let columns: Vec<DBCol> = DBCol::iter().collect();
        Self::open_with_columns(path, store_config, mode, temp, &columns)
    }

    /// Opens the database with given set of column families configured.
    ///
    /// With cold storage, we will need to be able to configure the database
    /// with only a subset of columns.  The `columns` argument specifies which
    /// columns to configure in the database.
    ///
    /// Note that RocksDB is weird.  It’s not possible to open database in
    /// read/write mode without specifying all the column families existing in
    /// the database.  On the other hand, it’s not possible to open database in
    /// read-only mode while specifying column families which don’t exist.
    ///
    /// Furthermore, note that when opening in read/write mode, we configure
    /// RocksDB to create missing columns.
    ///
    /// With all that, it’s actually quite messy if at some point we’ll end up
    /// opening cold storage as hot since it’ll create all the missing columns.
    fn open_with_columns(
        path: &Path,
        store_config: &StoreConfig,
        mode: Mode,
        temp: Temperature,
        columns: &[DBCol],
    ) -> io::Result<Self> {
print_file_path_and_function_name!();

        let counter = instance_tracker::InstanceTracker::try_new(store_config.max_open_files)
            .map_err(other_error)?;
        println!("----------let (db, db_opt) = Self::open_db(path, store_config, mode, temp, columns)?;시작----------");
        let (db, db_opt) = Self::open_db(path, store_config, mode, temp, columns)?;
        println!("----------let (db, db_opt) = Self::open_db(path, store_config, mode, temp, columns)?;끝----------");
        println!("----------let cf_handles = Self::get_cf_handles(&db, columns);시작----------");
        let cf_handles = Self::get_cf_handles(&db, columns);
        println!("----------let cf_handles = Self::get_cf_handles(&db, columns);끝----------");
        Ok(Self { db, db_opt, cf_handles, _instance_tracker: counter })
    }

    /// Opens the database with given column families configured.
    fn open_db(
        path: &Path,
        store_config: &StoreConfig,
        mode: Mode,
        temp: Temperature,
        columns: &[DBCol],
    ) -> io::Result<(DB, Options)> {
print_file_path_and_function_name!();
        println!("지정된 열 패밀리가 구성된 데이터베이스를 엽니다.");
        println!("let options = rocksdb_options(store_config, mode); 시작 ");
        let options = rocksdb_options(store_config, mode);
        println!("let options = rocksdb_options(store_config, mode); 끝 ");
        let cf_descriptors = columns
            .iter()
            .copied()
            .map(|col| {
                rocksdb::ColumnFamilyDescriptor::new(
                    col_name(col),
                    rocksdb_column_options(col, store_config, temp),
                )
            })
            .collect::<Vec<_>>();

        let db = if mode.read_only() {
            DB::open_cf_descriptors_read_only(&options, path, cf_descriptors, false)
        } else {
            DB::open_cf_descriptors(&options, path, cf_descriptors)
        }
        .map_err(into_other)?;
        if cfg!(feature = "single_thread_rocksdb") {
            // These have to be set after open db
            let mut env = Env::default().unwrap();
            env.set_bottom_priority_background_threads(0);
            env.set_high_priority_background_threads(0);
            env.set_low_priority_background_threads(0);
            env.set_background_threads(0);
            println!("Disabled all background threads in rocksdb");
        }
        Ok((db, options))
    }

    /// Returns mapping from [`DBCol`] to cf handle used with RocksDB calls.
    ///
    /// The mapping is created for column families given in the `columns` list
    /// only.  All other columns will map to `None`.
    ///
    /// ## Safety
    ///
    /// This function is safe but using the returned mapping safely requires
    /// that it does not outlive `db` and that `db` is not modified.  The safety
    /// relies on `db` returning stable mapping for column families.
    fn get_cf_handles(
        db: &DB,
        columns: &[DBCol],
    ) -> enum_map::EnumMap<DBCol, Option<std::ptr::NonNull<ColumnFamily>>> {
        print_file_path_and_function_name!();
        println!("let mut cf_handles = enum_map::EnumMap::default();");
        let mut cf_handles = enum_map::EnumMap::default();
        println!("for col in columns.iter().copied() {{");
        for col in columns.iter().copied() {
            let ptr = db
                .cf_handle(&col_name(col))
                .and_then(|cf| std::ptr::NonNull::new(cf as *const _ as *mut _))
                .unwrap_or_else(|| panic!("Missing cf handle for {col}"));
            cf_handles[col] = Some(ptr);
        }
        println!("}} // for 문 끝 ");
        for handle in cf_handles.iter() {
            println!("cf_handles::EnumMap :{:?}", handle);
        }
        cf_handles
    }

    /// Returns column family handler to use with RocsDB for given column.
    ///
    /// If the database has not been setup to access given column, panics if
    /// debug assertions are enabled or returns an error otherwise.
    ///
    /// ## Safety
    ///
    /// This function is safe so long as `db` field has not been modified since
    /// `cf_handles` mapping has been constructed.  We technically should mark
    /// this function unsafe but to improve ergonomy we didn’t.  This is an
    /// internal method so hopefully the implementation knows what it’s doing.
    fn cf_handle(&self, col: DBCol) -> io::Result<&ColumnFamily> {
print_file_path_and_function_name!();

        if let Some(ptr) = self.cf_handles[col] {
            // SAFETY: The pointers are valid so long as self.db is valid.
            Ok(unsafe { ptr.as_ref() })
        } else if cfg!(debug_assertions) {
            panic!("The database instance isn’t setup to access {col}");
        } else {
            Err(other_error(format!("{col}: no such column")))
        }
    }

    /// Returns iterator over all column families and their handles.
    ///
    /// This is kind of like iterating over all [`DBCol`] variants and calling
    /// [`Self::cf_handle`] except this method takes care of properly filtering
    /// out column families that the database instance isn’t setup to handle.
    ///
    /// ## Safety
    ///
    /// This function is safe so long as `db` field has not been modified since
    /// `cf_handles` mapping has been constructed.  We technically should mark
    /// this function unsafe but to improve ergonomy we didn’t.  This is an
    /// internal method so hopefully the implementation knows what it’s doing.
    fn cf_handles(&self) -> impl Iterator<Item = (DBCol, &ColumnFamily)> {
print_file_path_and_function_name!();
println!("RocksDB 호출에 사용되는 cf 핸들에 대한 매핑을 반환");
        self.cf_handles.iter().filter_map(|(col, ptr)| {
            if let Some(ptr) = *ptr {
                // SAFETY: The pointers are valid so long as self.db is valid.
                Some((col, unsafe { ptr.as_ref() }))
            } else {
                None
            }
        })
    }

    /// Iterates over rocksDB storage.
    /// You can optionally specify the bounds to limit the range over which it will iterate.
    /// You can specify EITHER the prefix, OR lower/upper bounds.
    /// Upper bound value is not included in the iteration.
    /// Specifying both prefix and lower & upper bounds will result in undetermined behavior.
    fn iter_raw_bytes_internal<'a>(
        &'a self,
        col: DBCol,
        prefix: Option<&'a [u8]>,
        lower_bound: Option<&'a [u8]>,
        upper_bound: Option<&'a [u8]>,
    ) -> RocksDBIterator<'a> {
print_file_path_and_function_name!();

        let cf_handle = self.cf_handle(col).unwrap();
        let mut read_options = rocksdb_read_options();
        if prefix.is_some() && (lower_bound.is_some() || upper_bound.is_some()) {
            panic!("Cannot iterate both with prefix and lower/upper bounds at the same time.");
        }
        if let Some(prefix) = prefix {
            read_options.set_iterate_range(::rocksdb::PrefixRange(prefix));
            // Note: prefix_same_as_start doesn’t do anything for us.  It takes
            // effect only if prefix extractor is configured for the column
            // family which is something we’re not doing.  Setting this option
            // is therefore pointless.
            //     read_options.set_prefix_same_as_start(true);
        }
        if let Some(lower_bound) = lower_bound {
            read_options.set_iterate_lower_bound(lower_bound);
        }
        if let Some(upper_bound) = upper_bound {
            read_options.set_iterate_upper_bound(upper_bound);
        }
        let iter = self.db.iterator_cf_opt(cf_handle, read_options, IteratorMode::Start);
        RocksDBIterator(iter)
    }
}

struct RocksDBIterator<'a>(rocksdb::DBIteratorWithThreadMode<'a, DB>);

impl<'a> Iterator for RocksDBIterator<'a> {
    type Item = io::Result<(Box<[u8]>, Box<[u8]>)>;

    fn next(&mut self) -> Option<Self::Item> {
print_file_path_and_function_name!();

        Some(self.0.next()?.map_err(into_other))
    }
}

impl<'a> std::iter::FusedIterator for RocksDBIterator<'a> {}

impl RocksDB {
    /// Returns ranges of keys in a given column family.
    ///
    /// In other words, returns the smallest and largest key in the column.  If
    /// the column is empty, returns `None`.
    fn get_cf_key_range(
        &self,
        cf_handle: &ColumnFamily,
    ) -> Result<Option<std::ops::RangeInclusive<Box<[u8]>>>, ::rocksdb::Error> {
print_file_path_and_function_name!();

        let range = {
            let mut iter = self.db.iterator_cf(cf_handle, IteratorMode::Start);
            let start = iter.next().transpose()?;
            iter.set_mode(IteratorMode::End);
            let end = iter.next().transpose()?;
            (start, end)
        };
        match range {
            (Some(start), Some(end)) => Ok(Some(start.0..=end.0)),
            (None, None) => Ok(None),
            _ => unreachable!(),
        }
    }
}

impl Database for RocksDB {
    fn get_raw_bytes(&self, col: DBCol, key: &[u8]) -> io::Result<Option<DBSlice<'_>>> {
print_file_path_and_function_name!();

        let timer =
            metrics::DATABASE_OP_LATENCY_HIST.with_label_values(&["get", col.into()]).start_timer();
        let read_options = rocksdb_read_options();
        let result = self
            .db
            .get_pinned_cf_opt(self.cf_handle(col)?, key, &read_options)
            .map_err(into_other)?
            .map(DBSlice::from_rocksdb_slice);
        timer.observe_duration();
        Ok(result)
    }

    fn iter_raw_bytes<'a>(&'a self, col: DBCol) -> DBIterator<'a> {
print_file_path_and_function_name!();

        Box::new(self.iter_raw_bytes_internal(col, None, None, None))
    }

    fn iter<'a>(&'a self, col: DBCol) -> DBIterator<'a> {
print_file_path_and_function_name!();

        refcount::iter_with_rc_logic(col, self.iter_raw_bytes_internal(col, None, None, None))
    }

    fn iter_prefix<'a>(&'a self, col: DBCol, key_prefix: &'a [u8]) -> DBIterator<'a> {
print_file_path_and_function_name!();

        let iter = self.iter_raw_bytes_internal(col, Some(key_prefix), None, None);
        refcount::iter_with_rc_logic(col, iter)
    }

    fn iter_range<'a>(
        &'a self,
        col: DBCol,
        lower_bound: Option<&'a [u8]>,
        upper_bound: Option<&'a [u8]>,
    ) -> DBIterator<'a> {
print_file_path_and_function_name!();

        let iter = self.iter_raw_bytes_internal(col, None, lower_bound, upper_bound);
        refcount::iter_with_rc_logic(col, iter)
    }

    fn write(&self, transaction: DBTransaction) -> io::Result<()> {
print_file_path_and_function_name!();

        let mut batch = WriteBatch::default();
        for op in transaction.ops {
            match op {
                DBOp::Set { col, key, value } => {
                    batch.put_cf(self.cf_handle(col)?, key, value);
                }
                DBOp::Insert { col, key, value } => {
                    if cfg!(debug_assertions) {
                        if let Ok(Some(old_value)) = self.get_raw_bytes(col, &key) {
                            super::assert_no_overwrite(col, &key, &value, &*old_value)
                        }
                    }
                    batch.put_cf(self.cf_handle(col)?, key, value);
                }
                DBOp::UpdateRefcount { col, key, value } => {
                    batch.merge_cf(self.cf_handle(col)?, key, value);
                }
                DBOp::Delete { col, key } => {
                    batch.delete_cf(self.cf_handle(col)?, key);
                }
                DBOp::DeleteAll { col } => {
                    let cf_handle = self.cf_handle(col)?;
                    let range = self.get_cf_key_range(cf_handle).map_err(into_other)?;
                    if let Some(range) = range {
                        batch.delete_range_cf(cf_handle, range.start(), range.end());
                        // delete_range_cf deletes ["begin_key", "end_key"), so need one more delete
                        batch.delete_cf(cf_handle, range.end())
                    }
                }
                DBOp::DeleteRange { col, from, to } => {
                    batch.delete_range_cf(self.cf_handle(col)?, from, to);
                }
            }
        }
        self.db.write(batch).map_err(into_other)
    }

    fn compact(&self) -> io::Result<()> {
print_file_path_and_function_name!();

        let none = Option::<&[u8]>::None;
        for col in DBCol::iter() {
            self.db.compact_range_cf(self.cf_handle(col)?, none, none);
        }
        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
print_file_path_and_function_name!();

        // Need to iterator over all CFs because the normal `flush()` only
        // flushes the default column family.
        for col in DBCol::iter() {
            self.db.flush_cf(self.cf_handle(col)?).map_err(into_other)?;
        }
        Ok(())
    }

    /// Trying to get
    /// 1. RocksDB statistics - RocksDB 통계
    /// 2. Selected RockdDB properties for column families - column families 에 대해 선택된 RockdDB 속성
    fn get_store_statistics(&self) -> Option<StoreStatistics> {
        println!("Trying to get \n 1. RocksDB statistics - RocksDB 통계\n2. Selected RockdDB properties for column families - column families 에 대해 선택된 RockdDB 속성");
        print_file_path_and_function_name!();


        let mut result = StoreStatistics { data: vec![] };
        if let Some(stats_str) = self.db_opt.get_statistics() {
            if let Err(err) = parse_statistics(&stats_str, &mut result) {
                warn!(target: "store", "Failed to parse store statistics: {:?}", err);
            }
        }
        self.get_cf_statistics(&mut result);
        if result.data.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    fn create_checkpoint(&self, path: &std::path::Path) -> anyhow::Result<()> {
print_file_path_and_function_name!();

        let cp = ::rocksdb::checkpoint::Checkpoint::new(&self.db)?;
        cp.create_checkpoint(path)?;
        Ok(())
    }
}

/// DB level options
fn rocksdb_options(store_config: &StoreConfig, mode: Mode) -> Options {
print_file_path_and_function_name!();
println!(" DB level options");
    let mut opts = Options::default();

    set_compression_options(&mut opts);
    opts.create_missing_column_families(mode.read_write());
    opts.create_if_missing(mode.can_create());
    opts.set_use_fsync(false);
    opts.set_max_open_files(store_config.max_open_files.try_into().unwrap_or(i32::MAX));
    opts.set_keep_log_file_num(1);
    opts.set_bytes_per_sync(bytesize::MIB);
    opts.set_write_buffer_size(256 * bytesize::MIB as usize);
    opts.set_max_bytes_for_level_base(256 * bytesize::MIB);

    if cfg!(feature = "single_thread_rocksdb") {
        opts.set_disable_auto_compactions(true);
        opts.set_max_background_jobs(0);
        opts.set_stats_dump_period_sec(0);
        opts.set_stats_persist_period_sec(0);
        opts.set_level_zero_slowdown_writes_trigger(-1);
        opts.set_level_zero_file_num_compaction_trigger(-1);
        opts.set_level_zero_stop_writes_trigger(100000000);
    } else {
        opts.increase_parallelism(std::cmp::max(1, num_cpus::get() as i32 / 2));
        opts.set_max_total_wal_size(bytesize::GIB);
    }

    // TODO(mina86): Perhaps enable statistics even in read-only mode?
    if mode.read_write() && store_config.enable_statistics {
        // Rust API doesn't permit choosing stats level. The default stats level
        // is `kExceptDetailedTimers`, which is described as: "Collects all
        // stats except time inside mutex lock AND time spent on compression."
        opts.enable_statistics();
        // Disabling dumping stats to files because the stats are exported to
        // Prometheus.
        opts.set_stats_persist_period_sec(0);
        opts.set_stats_dump_period_sec(0);
    }
    println!("--------------------------------rocksdb_options 끝 --------------------------------");
    opts
}

fn rocksdb_read_options() -> ReadOptions {
print_file_path_and_function_name!();

    let mut read_options = ReadOptions::default();
    read_options.set_verify_checksums(false);
    read_options
}

fn rocksdb_block_based_options(
    block_size: bytesize::ByteSize,
    cache_size: bytesize::ByteSize,
) -> BlockBasedOptions {
//print_file_path_and_function_name!();

    let mut block_opts = BlockBasedOptions::default();
    block_opts.set_block_size(block_size.as_u64().try_into().unwrap());
    // We create block_cache for each of 47 columns, so the total cache size is 32 * 47 = 1504mb
    block_opts
        .set_block_cache(&Cache::new_lru_cache(cache_size.as_u64().try_into().unwrap()).unwrap());
    block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
    block_opts.set_cache_index_and_filter_blocks(true);
    block_opts.set_bloom_filter(10.0, true);
    block_opts
}

fn rocksdb_column_options(col: DBCol, store_config: &StoreConfig, temp: Temperature) -> Options {
//print_file_path_and_function_name!();

    let mut opts = Options::default();
    set_compression_options(&mut opts);
    opts.set_level_compaction_dynamic_level_bytes(true);
    let cache_size = store_config.col_cache_size(col);
    opts.set_block_based_table_factory(&rocksdb_block_based_options(
        store_config.block_size,
        cache_size,
    ));

    // Note that this function changes a lot of rustdb parameters including:
    //      write_buffer_size = memtable_memory_budget / 4
    //      min_write_buffer_number_to_merge = 2
    //      max_write_buffer_number = 6
    //      level0_file_num_compaction_trigger = 2
    //      target_file_size_base = memtable_memory_budget / 8
    //      max_bytes_for_level_base = memtable_memory_budget
    //      compaction_style = kCompactionStyleLevel
    // Also it sets compression_per_level in a way that the first 2 levels have no compression and
    // the rest use LZ4 compression.
    // See the implementation here:
    //      https://github.com/facebook/rocksdb/blob/c18c4a081c74251798ad2a1abf83bad417518481/options/options.cc#L588.
    let memtable_memory_budget = 128 * bytesize::MIB as usize;
    opts.optimize_level_style_compaction(memtable_memory_budget);

    opts.set_target_file_size_base(64 * bytesize::MIB);
    if temp == Temperature::Hot && col.is_rc() {
        opts.set_merge_operator("refcount merge", RocksDB::refcount_merge, RocksDB::refcount_merge);
        opts.set_compaction_filter("empty value filter", RocksDB::empty_value_compaction_filter);
    }
    opts
}

fn set_compression_options(opts: &mut Options) {
//print_file_path_and_function_name!();

    opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
    opts.set_bottommost_compression_type(rocksdb::DBCompressionType::Zstd);
    // RocksDB documenation says that 16KB is a typical dictionary size.
    // We've empirically tuned the dicionary size to twice of that 'typical' size.
    // Having train data size x100 from dictionary size is a recommendation from RocksDB.
    // See: https://rocksdb.org/blog/2021/05/31/dictionary-compression.html?utm_source=dbplatz
    let dict_size = 2 * 16384;
    let max_train_bytes = dict_size * 100;
    // We use default parameters of RocksDB here:
    //      window_bits is -14 and is unused (Zlib-specific parameter),
    //      compression_level is 32767 meaning the default compression level for ZSTD,
    //      compression_strategy is 0 and is unused (Zlib-specific parameter).
    // See: https://github.com/facebook/rocksdb/blob/main/include/rocksdb/advanced_options.h#L176:
    opts.set_bottommost_compression_options(
        /*window_bits */ -14, /*compression_level */ 32767,
        /*compression_strategy */ 0, dict_size, /*enabled */ true,
    );
    opts.set_bottommost_zstd_max_train_bytes(max_train_bytes, true);
}

impl RocksDB {
    /// Blocks until all RocksDB instances (usually 0 or 1) gracefully shutdown.
    pub fn block_until_all_instances_are_dropped() {
print_file_path_and_function_name!();

        instance_tracker::block_until_all_instances_are_closed();
    }

    /// Returns metadata of the database or `None` if the db doesn’t exist.
    pub(crate) fn get_metadata(
        path: &Path,
        config: &StoreConfig,
    ) -> io::Result<Option<metadata::DbMetadata>> {
print_file_path_and_function_name!();

        if !path.join("CURRENT").is_file() {
            return Ok(None);
        }
        // Specify only DBCol::DbVersion.  It’s ok to open db in read-only mode
        // without specifying all column families but it’s an error to provide
        // a descriptor for a column family which doesn’t exist.  This allows us
        // to read the version without modifying the database before we figure
        // out if there are any necessary migrations to perform.
        let cols = [DBCol::DbVersion];
        let db = Self::open_with_columns(path, config, Mode::ReadOnly, Temperature::Hot, &cols)?;
        Some(metadata::DbMetadata::read(&db)).transpose()
    }

    /// Gets every int property in CF_PROPERTY_NAMES for every column in DBCol.
    /// DB Col의 모든 열에 대해 CF_PROPERTY_NAMES 의 모든 int 속성을 가져옵니다.
    fn get_cf_statistics(&self, result: &mut StoreStatistics) {
        print_file_path_and_function_name!();
        println!("DB Col의 모든 열에 대해 CF_PROPERTY_NAMES 의 모든 int 속성을 가져옵니다.");
        for prop_name in CF_PROPERTY_NAMES {
            let values = self
                .cf_handles()
                .filter_map(|(col, handle)| {
                    let prop = self.db.property_int_value_cf(handle, prop_name);
                    Some(StatsValue::ColumnValue(col, prop.ok()?? as i64))
                })
                .collect::<Vec<_>>();
            if !values.is_empty() {
                // TODO(mina86): Once const_str_from_utf8 is stabilised we might
                // be able convert this runtime UTF-8 validation into const.
                let stat_name = prop_name.to_str().unwrap();
                result.data.push((stat_name.to_string(), values));
            }
        }
    }
}

impl Drop for RocksDB {
    fn drop(&mut self) {
print_file_path_and_function_name!();

        if cfg!(feature = "single_thread_rocksdb") {
            // RocksDB with only one thread stuck on wait some condition var
            // Turn on additional threads to proceed
            let mut env = Env::default().unwrap();
            env.set_background_threads(4);
        }
        self.db.cancel_all_background_work(true);
    }
}

/// Parses a string containing RocksDB statistics.
/// Results are added into provided 'result' parameter.
fn parse_statistics(
    statistics: &str,
    result: &mut StoreStatistics,
) -> Result<(), Box<dyn std::error::Error>> {
print_file_path_and_function_name!();

    // Statistics are given one per line.
    for line in statistics.lines() {
        // Each line follows one of two formats:
        // 1) <stat_name> COUNT : <value>
        // 2) <stat_name> P50 : <value> P90 : <value> COUNT : <value> SUM : <value>
        // Each line gets split into words and we parse statistics according to this format.
        if let Some((stat_name, words)) = line.split_once(' ') {
            let mut values = vec![];
            let mut words = words.split(" : ").flat_map(|v| v.split(" "));
            while let (Some(key), Some(val)) = (words.next(), words.next()) {
                match key {
                    "COUNT" => values.push(StatsValue::Count(val.parse::<i64>()?)),
                    "SUM" => values.push(StatsValue::Sum(val.parse::<i64>()?)),
                    p if p.starts_with("P") => values.push(StatsValue::Percentile(
                        key[1..].parse::<u32>()?,
                        val.parse::<f64>()?,
                    )),
                    _ => {
                        warn!(target: "stats", "Unsupported stats value: {key} in {line}");
                    }
                }
            }
            // We push some stats to result, even if later parsing will fail.
            result.data.push((stat_name.to_string(), values));
        }
    }
    Ok(())
}

fn other_error(msg: String) -> io::Error {
print_file_path_and_function_name!();

    io::Error::new(io::ErrorKind::Other, msg)
}

fn into_other(error: rocksdb::Error) -> io::Error {
print_file_path_and_function_name!();

    io::Error::new(io::ErrorKind::Other, error.into_string())
}

/// Returns name of a RocksDB column family corresponding to given column.
///
/// Historically we used `col##` names (with `##` being index of the column).
/// We have since deprecated this convention.  All future column families are
/// named the same as the variant of the [`DBCol`] enum.
fn col_name(col: DBCol) -> &'static str {
//print_file_path_and_function_name!(); 개많음

    match col {
        DBCol::DbVersion => "col0",
        DBCol::BlockMisc => "col1",
        DBCol::Block => "col2",
        DBCol::BlockHeader => "col3",
        DBCol::BlockHeight => "col4",
        DBCol::State => "col5",
        DBCol::ChunkExtra => "col6",
        DBCol::_TransactionResult => "col7",
        DBCol::OutgoingReceipts => "col8",
        DBCol::IncomingReceipts => "col9",
        DBCol::_Peers => "col10",
        DBCol::EpochInfo => "col11",
        DBCol::BlockInfo => "col12",
        DBCol::Chunks => "col13",
        DBCol::PartialChunks => "col14",
        DBCol::BlocksToCatchup => "col15",
        DBCol::StateDlInfos => "col16",
        DBCol::ChallengedBlocks => "col17",
        DBCol::StateHeaders => "col18",
        DBCol::InvalidChunks => "col19",
        DBCol::BlockExtra => "col20",
        DBCol::BlockPerHeight => "col21",
        DBCol::StateParts => "col22",
        DBCol::EpochStart => "col23",
        DBCol::AccountAnnouncements => "col24",
        DBCol::NextBlockHashes => "col25",
        DBCol::EpochLightClientBlocks => "col26",
        DBCol::ReceiptIdToShardId => "col27",
        DBCol::_NextBlockWithNewChunk => "col28",
        DBCol::_LastBlockWithNewChunk => "col29",
        DBCol::PeerComponent => "col30",
        DBCol::ComponentEdges => "col31",
        DBCol::LastComponentNonce => "col32",
        DBCol::Transactions => "col33",
        DBCol::_ChunkPerHeightShard => "col34",
        DBCol::StateChanges => "col35",
        DBCol::BlockRefCount => "col36",
        DBCol::TrieChanges => "col37",
        DBCol::BlockMerkleTree => "col38",
        DBCol::ChunkHashesByHeight => "col39",
        DBCol::BlockOrdinal => "col40",
        DBCol::_GCCount => "col41",
        DBCol::OutcomeIds => "col42",
        DBCol::_TransactionRefCount => "col43",
        DBCol::ProcessedBlockHeights => "col44",
        DBCol::Receipts => "col45",
        DBCol::CachedContractCode => "col46",
        DBCol::EpochValidatorInfo => "col47",
        DBCol::HeaderHashesByHeight => "col48",
        DBCol::StateChangesForSplitStates => "col49",
        // If you’re adding a new column, do *not* create a new case for it.
        // All new columns are handled by this default case:
        #[allow(unreachable_patterns)]
        _ => <&str>::from(col),
    }
}

#[cfg(test)]
mod tests {
    use crate::db::{Database, StatsValue};
    use crate::{DBCol, NodeStorage, StoreStatistics};
    use assert_matches::assert_matches;

    use super::*;

    #[test]
    fn rocksdb_merge_sanity() {
        let (_tmp_dir, opener) = NodeStorage::test_opener();
        let store = opener.open().unwrap().get_hot_store();
        let ptr = (&*store.storage) as *const (dyn Database + 'static);
        let rocksdb = unsafe { &*(ptr as *const RocksDB) };
        assert_eq!(store.get(DBCol::State, &[1]).unwrap(), None);
        {
            let mut store_update = store.store_update();
            store_update.increment_refcount(DBCol::State, &[1], &[1]);
            store_update.commit().unwrap();
        }
        {
            let mut store_update = store.store_update();
            store_update.increment_refcount(DBCol::State, &[1], &[1]);
            store_update.commit().unwrap();
        }
        assert_eq!(store.get(DBCol::State, &[1]).unwrap().as_deref(), Some(&[1][..]));
        assert_eq!(
            rocksdb.get_raw_bytes(DBCol::State, &[1]).unwrap().as_deref(),
            Some(&[1, 2, 0, 0, 0, 0, 0, 0, 0][..])
        );
        {
            let mut store_update = store.store_update();
            store_update.decrement_refcount(DBCol::State, &[1]);
            store_update.commit().unwrap();
        }
        assert_eq!(store.get(DBCol::State, &[1]).unwrap().as_deref(), Some(&[1][..]));
        assert_eq!(
            rocksdb.get_raw_bytes(DBCol::State, &[1]).unwrap().as_deref(),
            Some(&[1, 1, 0, 0, 0, 0, 0, 0, 0][..])
        );
        {
            let mut store_update = store.store_update();
            store_update.decrement_refcount(DBCol::State, &[1]);
            store_update.commit().unwrap();
        }
        // Refcount goes to 0 -> get() returns None
        assert_eq!(store.get(DBCol::State, &[1]).unwrap(), None);
        // Internally there is an empty value
        assert_eq!(rocksdb.get_raw_bytes(DBCol::State, &[1]).unwrap().as_deref(), Some(&[][..]));

        // single_thread_rocksdb makes compact hang forever
        if !cfg!(feature = "single_thread_rocksdb") {
            let none = Option::<&[u8]>::None;
            let cf = rocksdb.cf_handle(DBCol::State).unwrap();

            // I’m not sure why but we need to run compaction twice.  If we run
            // it only once, we end up with an empty value for the key.  This is
            // surprising because I assumed that compaction filter would discard
            // empty values.
            rocksdb.db.compact_range_cf(cf, none, none);
            assert_eq!(
                rocksdb.get_raw_bytes(DBCol::State, &[1]).unwrap().as_deref(),
                Some(&[][..])
            );
            assert_eq!(store.get(DBCol::State, &[1]).unwrap(), None);

            rocksdb.db.compact_range_cf(cf, none, none);
            assert_eq!(rocksdb.get_raw_bytes(DBCol::State, &[1]).unwrap(), None);
            assert_eq!(store.get(DBCol::State, &[1]).unwrap(), None);
        }
    }

    #[test]
    fn test_parse_statistics() {
        let statistics = "rocksdb.cold.file.read.count COUNT : 999\n\
         rocksdb.db.get.micros P50 : 9.171086 P95 : 222.678751 P99 : 549.611652 P100 : 45816.000000 COUNT : 917578 SUM : 38313754";
        let mut result = StoreStatistics { data: vec![] };
        let parse_result = parse_statistics(statistics, &mut result);
        // We should be able to parse stats and the result should be Ok(()).
        parse_result.unwrap();
        assert_eq!(
            result,
            StoreStatistics {
                data: vec![
                    ("rocksdb.cold.file.read.count".to_string(), vec![StatsValue::Count(999)]),
                    (
                        "rocksdb.db.get.micros".to_string(),
                        vec![
                            StatsValue::Percentile(50, 9.171086),
                            StatsValue::Percentile(95, 222.678751),
                            StatsValue::Percentile(99, 549.611652),
                            StatsValue::Percentile(100, 45816.0),
                            StatsValue::Count(917578),
                            StatsValue::Sum(38313754)
                        ]
                    )
                ]
            }
        );
    }

    #[test]
    fn test_delete_range() {
        let store = NodeStorage::test_opener().1.open().unwrap().get_hot_store();
        let keys = [vec![0], vec![1], vec![2], vec![3]];
        let column = DBCol::Block;

        let mut store_update = store.store_update();
        for key in &keys {
            store_update.insert(column, key, &vec![42]);
        }
        store_update.commit().unwrap();

        let mut store_update = store.store_update();
        store_update.delete_range(column, &keys[1], &keys[3]);
        store_update.commit().unwrap();

        assert_matches!(store.exists(column, &keys[0]), Ok(true));
        assert_matches!(store.exists(column, &keys[1]), Ok(false));
        assert_matches!(store.exists(column, &keys[2]), Ok(false));
        assert_matches!(store.exists(column, &keys[3]), Ok(true));
    }
}