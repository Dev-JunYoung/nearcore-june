use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


pub use crate::config::{init_configs, load_config, load_test_config, NearConfig, NEAR_BASE};
pub use crate::runtime::NightshadeRuntime;

use crate::cold_storage::spawn_cold_store_loop;
use crate::state_sync::{spawn_state_sync_dump, StateSyncDumpHandle};
use actix::{Actor, Addr};
use actix_rt::ArbiterHandle;
use anyhow::Context;
use cold_storage::ColdStoreLoopHandle;
use near_async::actix::AddrWithAutoSpanContextExt;
use near_async::messaging::{IntoSender, LateBoundSender};
use near_async::time;
use near_chain::{Chain, ChainGenesis};
use near_chunks::shards_manager_actor::start_shards_manager;
use near_client::{start_client, start_view_client, ClientActor, ConfigUpdater, ViewClientActor};
use near_network::PeerManagerActor;
use near_primitives::block::GenesisId;
use near_store::metadata::DbKind;
use near_store::metrics::spawn_db_metrics_loop;
use near_store::{DBCol, Mode, NodeStorage, Store, StoreOpenerError};
use near_telemetry::TelemetryActor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

pub mod append_only_map;
mod cold_storage;
pub mod config;
mod config_validate;
mod download_file;
pub mod dyn_config;
mod metrics;
pub mod migrations;
mod runtime;
pub mod state_sync;

pub fn get_default_home() -> PathBuf {
    print_file_path_and_function_name!();

    if let Ok(near_home) = std::env::var("NEAR_HOME") {
        return near_home.into();
    }

    if let Some(mut home) = dirs::home_dir() {
        home.push(".near");
        return home;
    }

    PathBuf::default()
}

/// Opens node’s storage performing migrations and checks when necessary.
///
/// If opened storage is an RPC store and `near_config.config.archive` is true,
/// converts the storage to archival node.  Otherwise, if opening archival node
/// with that field being false, prints a warning and sets the field to `true`.
/// In other words, once store is archival, the node will act as archival nod
/// regardless of settings in `config.json`.
///
/// The end goal is to get rid of `archive` option in `config.json` file and
/// have the type of the node be determined purely based on kind of database
/// being opened.
///
/// 마이그레이션을 수행하는 노드의 스토리지를 열고 필요한 경우 확인합니다.///
/// 열린 저장소가 RPC 저장소이고 `near_config.config.archive`가 true 인 경우,
/// 스토리지를 아카이브 노드로 변환합니다.  그렇지 않으면, 해당 필드가 거짓인 상태에서 아카이브 노드를 열면
/// 해당 필드가 거짓이면 /// 경고를 출력하고 필드를 `true`로 설정합니다.
/// 즉, 일단 저장소가 아카이브 노드로 전환되면, 노드는 아카이브 노드로 동작합니다.
/// 노드로 동작합니다.
///
/// 최종 목표는 `config.json` 파일에서 `archive` 옵션을 없애고
/// 노드 유형이 순전히 열려있는 데이터베이스의 종류에 따라 결정되도록 하는 것입니다.
/// 노드 유형을 순수하게 데이터베이스 종류에 따라 결정하도록 하는 것입니다.
fn open_storage(home_dir: &Path, near_config: &mut NearConfig) -> anyhow::Result<NodeStorage> {
    print_file_path_and_function_name!();
    println!("----------let migrator = migrations::Migrator::new(near_config);시작----------");
    let migrator = migrations::Migrator::new(near_config);
    println!("----------let migrator = migrations::Migrator::new(near_config);끝----------");
    println!("----------let opener = NodeStorage::opener(----------");
    let opener = NodeStorage::opener(
        home_dir,
        near_config.client_config.archive,
        &near_config.config.store,
        near_config.config.cold_store.as_ref(),
    )
        .with_migrator(&migrator);
    println!(")
        .with_migrator(&migrator);");
    println!("let storage = match opener.open()");
    let storage = match opener.open() {
        Ok(storage) => Ok(storage),
        Err(StoreOpenerError::IO(err)) => {
            Err(anyhow::anyhow!("{err}"))
        }
        // Cannot happen with Mode::ReadWrite
        Err(StoreOpenerError::DbDoesNotExist) => unreachable!(),
        // Cannot happen with Mode::ReadWrite
        Err(StoreOpenerError::DbAlreadyExists) => unreachable!(),
        Err(StoreOpenerError::HotColdExistenceMismatch) => {
            Err(anyhow::anyhow!(
                "Hot and cold databases must either both exist or both not exist.\n\
                 Note that at this moment it’s not possible to convert and RPC or legacy archive database into split hot+cold database.\n\
                 To set up node in that configuration, start with neither of the databases existing.",
            ))
        }
        Err(err @ StoreOpenerError::HotColdVersionMismatch { .. }) => {
            Err(anyhow::anyhow!("{err}"))
        }
        Err(StoreOpenerError::DbKindMismatch { which, got, want }) => {
            Err(if let Some(got) = got {
                anyhow::anyhow!("{which} database kind should be {want} but got {got}")
            } else {
                anyhow::anyhow!("{which} database kind should be {want} but none was set")
            })
        }
        Err(StoreOpenerError::SnapshotAlreadyExists(snap_path)) => {
            Err(anyhow::anyhow!(
                "Detected an existing database migration snapshot at ‘{}’.\n\
                 Probably a database migration got interrupted and your database is corrupted.\n\
                 Please replace files in ‘{}’ with contents of the snapshot, delete the snapshot and try again.",
                snap_path.display(),
                opener.path().display(),
            ))
        }
        Err(StoreOpenerError::SnapshotError(err)) => {
            use near_store::config::MigrationSnapshot;
            let path = std::path::PathBuf::from("/path/to/snapshot/dir");
            let on = MigrationSnapshot::Path(path).format_example();
            let off = MigrationSnapshot::Enabled(false).format_example();
            Err(anyhow::anyhow!(
                "Failed to create a database migration snapshot: {err}.\n\
                 To change the location of snapshot adjust \
                 ‘store.migration_snapshot’ property in ‘config.json’:\n{on}\n\
                 Alternatively, you can disable database migration snapshots \
                 in `config.json`:\n{off}"
            ))
        }
        Err(StoreOpenerError::SnapshotRemoveError { path, error }) => {
            let path = path.display();
            Err(anyhow::anyhow!(
                "The DB migration has succeeded but deleting of the snapshot \
                 at {path} has failed: {error}\n
                 Try renaming the snapshot directory to temporary name (e.g. \
                 by adding tilde to its name) and starting the node.  If that \
                 works, the snapshot can be deleted."))
        }
        // Cannot happen with Mode::ReadWrite
        Err(StoreOpenerError::DbVersionMismatchOnRead { .. }) => unreachable!(),
        // Cannot happen when migrator is specified.
        Err(StoreOpenerError::DbVersionMismatch { .. }) => unreachable!(),
        Err(StoreOpenerError::DbVersionMissing { .. }) => {
            Err(anyhow::anyhow!("Database version is missing!"))
        }
        Err(StoreOpenerError::DbVersionTooOld { got, latest_release, .. }) => {
            Err(anyhow::anyhow!(
                "Database version {got} is created by an old version \
                 of neard and is no longer supported, please migrate using \
                 {latest_release} release"
            ))
        }
        Err(StoreOpenerError::DbVersionTooNew { got, want }) => {
            Err(anyhow::anyhow!(
                "Database version {got} is higher than the expected version {want}. \
                It was likely created by newer version of neard. Please upgrade your neard."
            ))
        }
        Err(StoreOpenerError::MigrationError(err)) => {
            Err(err)
        }
    }.with_context(|| format!("unable to open database at {}", opener.path().display()))?;
    println!("      -------------- let storage = match opener.open().with_context() 끝 ------------------------------------");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      --------------------------------------------------");
    println!("opener.path().display() :  {:#?}", opener.path().display());

    println!("\n--------------------------------------------------");
    println!("      -------------- near_config.config.archive = storage.is_archive()?; 시작 ------------------------------------");
    print_file_path_and_function_name!();
    near_config.config.archive = storage.is_archive()?;
    println!("      -------------- near_config.config.archive = storage.is_archive()?; 끝 ------------------------------------");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      --------------------------------------------------");
    println!("near_config.config.archive :  {:#?}", near_config.config.archive);
    println!("------------- open_storage() 끝!\n-------------------------------------");
    println!("\n--------------------------------------------------");

    Ok(storage)
}

// Safely get the split store while checking that all conditions to use it are met.
fn get_split_store(config: &NearConfig, storage: &NodeStorage) -> anyhow::Result<Option<Store>> {
    print_file_path_and_function_name!();

    // SplitStore should only be used on archival nodes.
    if !config.config.archive {
        return Ok(None);
    }

    // SplitStore should only be used if cold store is configured.
    if config.config.cold_store.is_none() {
        return Ok(None);
    }

    // SplitStore should only be used in the view client if it is enabled.
    if !config.config.split_storage.as_ref().map_or(false, |c| c.enable_split_storage_view_client) {
        return Ok(None);
    }

    // SplitStore should only be used if the migration is finished. The
    // migration to cold store is finished when the db kind of the hot store is
    // changed from Archive to Hot.
    if storage.get_hot_store().get_db_kind()? != Some(DbKind::Hot) {
        return Ok(None);
    }

    Ok(storage.get_split_store())
}

pub struct NearNode {
    pub client: Addr<ClientActor>,
    pub view_client: Addr<ViewClientActor>,
    pub arbiters: Vec<ArbiterHandle>,
    pub rpc_servers: Vec<(&'static str, actix_web::dev::ServerHandle)>,
    /// The cold_store_loop_handle will only be set if the cold store is configured.
    /// It's a handle to a background thread that copies data from the hot store to the cold store.
    pub cold_store_loop_handle: Option<ColdStoreLoopHandle>,
    /// Contains handles to background threads that may be dumping state to S3.
    pub state_sync_dump_handle: Option<StateSyncDumpHandle>,
}

//시작과 함께 설정파일 실행
pub fn start_with_config(home_dir: &Path, config: NearConfig) -> anyhow::Result<NearNode> {
    print_file_path_and_function_name!();

    start_with_config_and_synchronization(home_dir, config, None, None)
}

pub fn start_with_config_and_synchronization(
    home_dir: &Path,//
    mut config: NearConfig,//
    // 'shutdown_signal' will notify the corresponding `oneshot::Receiver` when an instance of
    // `ClientActor` gets dropped.
    shutdown_signal: Option<broadcast::Sender<()>>,//
    config_updater: Option<ConfigUpdater>,//
) -> anyhow::Result<NearNode> {
    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("    중요한 분기점 : start_with_config_and_synchronization() 시작");
    println!("let storage = open_storage(home_dir, &mut config)?;");
    let storage = open_storage(home_dir, &mut config)?; //DB : 1
    println!("let storage = open_storage(home_dir, &mut config)?;");
    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("let db_metrics_arbiter = if config.client_config.enable_statistics_export");
    let db_metrics_arbiter = if config.client_config.enable_statistics_export {
        println!("let period = config.client_config.log_summary_period;");
        let period = config.client_config.log_summary_period;

        println!("let db_metrics_arbiter_handle = spawn_db_metrics_loop(&storage, period)?;");
        let db_metrics_arbiter_handle = spawn_db_metrics_loop(&storage, period)?;

        println!("Some(db_metrics_arbiter_handle)");
        Some(db_metrics_arbiter_handle)
    } else {
        println!("None");
        None
    };
    println!("\n--------------------------------------------------");
    println!("db_metrics_arbiter :  {:#?}", db_metrics_arbiter);
    println!("--------------------------------------------------\n");
    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한 지점 : 나이트샤드 생성 시작 ");
    println!("let runtime = NightshadeRuntime::from_config(home_dir, storage.get_hot_store(), &config);");
    let runtime = NightshadeRuntime::from_config(home_dir, storage.get_hot_store(), &config);
    println!("--------------------------------------------------\n");


    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    // Get the split store. If split store is some then create a new runtime for
    // the view client. Otherwise just re-use the existing runtime.
    println!("let split_store = get_split_store(&config, &storage)?;");
    let split_store = get_split_store(&config, &storage)?;

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("let view_runtime = if let Some(split_store) = split_store {{
        NightshadeRuntime::from_config(home_dir, split_store, &config)
    }} else {{
        runtime.clone()
    }};");
    let view_runtime = if let Some(split_store) = &split_store {
        NightshadeRuntime::from_config(home_dir, split_store.clone(), &config)
    } else {
        runtime.clone()
    };

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("--------------------------------------------------\n");
    println!("let cold_store_loop_handle = spawn_cold_store_loop(&config, &storage, runtime.clone())?;");
    let cold_store_loop_handle = spawn_cold_store_loop(&config, &storage, runtime.clone())?;

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("let telemetry = TelemetryActor::new(config.telemetry_config.clone()).start();");
    let telemetry = TelemetryActor::new(config.telemetry_config.clone()).start();

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("let chain_genesis = ChainGenesis::new(&config.genesis);");
    let chain_genesis = ChainGenesis::new(&config.genesis);

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("      중요! 제네시스블록 생성 시작");
    println!("let genesis_block = Chain::make_genesis_block(&*runtime, &chain_genesis)?;");
    let genesis_block = Chain::make_genesis_block(&*runtime, &chain_genesis)?;

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("let genesis_id = GenesisId {{
        chain_id: config.client_config.chain_id.clone(),
        hash: *genesis_block.header().hash(),
    }};");
    let genesis_id = GenesisId {
        chain_id: config.client_config.chain_id.clone(),
        hash: *genesis_block.header().hash(),
    };
    println!("genesis_id :  {:#?}", genesis_id);
    println!("--------------------------------------------------\n");


    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("let node_id = config.network_config.node_id();");
    let node_id = config.network_config.node_id();
    println!("node_id :  {:#?}", node_id);
    println!("--------------------------------------------------\n");


    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("let network_adapter = Arc::new(LateBoundSender::default());");
    let network_adapter = Arc::new(LateBoundSender::default());
    // println!("network_adapter :  {:#?}", network_adapter);
    println!("--------------------------------------------------\n");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("let shards_manager_adapter = Arc::new(LateBoundSender::default());");
    let shards_manager_adapter = Arc::new(LateBoundSender::default());
    println!("--------------------------------------------------\n");


    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("let client_adapter_for_shards_manager = Arc::new(LateBoundSender::default());");
    let client_adapter_for_shards_manager = Arc::new(LateBoundSender::default());
    println!("--------------------------------------------------\n");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("let adv = near_client::adversarial::Controls::new(config.client_config.archive);");
    let adv = near_client::adversarial::Controls::new(config.client_config.archive);
    println!("--------------------------------------------------\n");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("let view_client = start_view_client(
        config.validator_signer.as_ref().map(|signer| signer.validator_id().clone()),
        chain_genesis.clone(),
        view_runtime,
        network_adapter.clone().into(),
        config.client_config.clone(),
        adv.clone(),
    );");
    let view_client = start_view_client(
        config.validator_signer.as_ref().map(|signer| signer.validator_id().clone()),
        chain_genesis.clone(),
        view_runtime,
        network_adapter.clone().into(),
        config.client_config.clone(),
        adv.clone(),
    );
    println!("view_client :  {:#?}", view_client);
    println!("--------------------------------------------------\n");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("let (client_actor, client_arbiter_handle) = start_client(
        config.client_config.clone(),
        chain_genesis.clone(),
        runtime.clone(),
        node_id,
        network_adapter.clone().into(),
        shards_manager_adapter.as_sender(),
        config.validator_signer.clone(),
        telemetry,
        shutdown_signal,
        adv,
        config_updater,
    );");
    let (client_actor, client_arbiter_handle) = start_client(
        config.client_config.clone(),
        chain_genesis.clone(),
        runtime.clone(),
        node_id,
        network_adapter.clone().into(),
        shards_manager_adapter.as_sender(),
        config.validator_signer.clone(),
        telemetry,
        shutdown_signal,
        adv,
        config_updater,
    );
    println!("client_actor :  {:#?}", client_actor);
    println!("--------------------------------------------------\n");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("client_adapter_for_shards_manager.bind(client_actor.clone().with_auto_span_context());");
    client_adapter_for_shards_manager.bind(client_actor.clone().with_auto_span_context());
    println!("--------------------------------------------------\n");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("let (shards_manager_actor, shards_manager_arbiter_handle) = start_shards_manager(
        runtime.clone(),
        network_adapter.as_sender(),
        client_adapter_for_shards_manager.as_sender(),
        config.validator_signer.as_ref().map(|signer| signer.validator_id().clone()),
        storage.get_hot_store(),
        config.client_config.chunk_request_retry_period,
    );");
    let (shards_manager_actor, shards_manager_arbiter_handle) = start_shards_manager(
        runtime.clone(),
        network_adapter.as_sender(),
        client_adapter_for_shards_manager.as_sender(),
        config.validator_signer.as_ref().map(|signer| signer.validator_id().clone()),
        split_store.unwrap_or(storage.get_hot_store()),
        config.client_config.chunk_request_retry_period,
    );
    println!("--------------------------------------------------\n");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("shards_manager_adapter.bind(shards_manager_actor);");
    shards_manager_adapter.bind(shards_manager_actor);
    println!("--------------------------------------------------\n");


    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("let state_sync_dump_handle = spawn_state_sync_dump(
        &config.client_config,
        chain_genesis,
        runtime,
        config.validator_signer.as_ref().map(|signer| signer.validator_id().clone()),
    )?;");
    let state_sync_dump_handle = spawn_state_sync_dump(
        &config.client_config,
        chain_genesis,
        runtime,
        config.validator_signer.as_ref().map(|signer| signer.validator_id().clone()),
    )?;
    println!("--------------------------------------------------\n");

    println!("let mut rpc_servers = Vec::new();");
    #[allow(unused_mut)]
        let mut rpc_servers = Vec::new();
    println!("--------------------------------------------------\n");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("let network_actor = PeerManagerActor::spawn(
        time::Clock::real(),
        storage.into_inner(near_store::Temperature::Hot),
        config.network_config,
        Arc::new(near_client::adapter::Adapter::new(client_actor.clone(), view_client.clone())),
        shards_manager_adapter.as_sender(),
        genesis_id,
    )
        .context(\"PeerManager::spawn()\")?;");
    let network_actor = PeerManagerActor::spawn(
        time::Clock::real(),
        storage.into_inner(near_store::Temperature::Hot),
        config.network_config,
        Arc::new(near_client::adapter::Adapter::new(client_actor.clone(), view_client.clone())),
        shards_manager_adapter.as_sender(),
        genesis_id,
    )
        .context("PeerManager::spawn()")?;
    println!("--------------------------------------------------\n");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("network_adapter.bind(network_actor.clone().with_auto_span_context());");
    network_adapter.bind(network_actor.clone().with_auto_span_context());
    println!("--------------------------------------------------\n");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("if let Some(rpc_config) = config.rpc_config {{
        rpc_servers.extend(near_jsonrpc::start_http(
            rpc_config,
            config.genesis.config.clone(),
            client_actor.clone(),
            view_client.clone(),
            Some(network_actor),
        ));
    }}");
    #[cfg(feature = "json_rpc")]
    if let Some(rpc_config) = config.rpc_config {
        rpc_servers.extend(near_jsonrpc::start_http(
            rpc_config,
            config.genesis.config.clone(),
            client_actor.clone(),
            view_client.clone(),
            Some(network_actor),
        ));
    }
    println!("--------------------------------------------------\n");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("    if let Some(rosetta_rpc_config) = config.rosetta_rpc_config {{
        rpc_servers.push((
            \"Rosetta RPC\",
            near_rosetta_rpc::start_rosetta_rpc(
                rosetta_rpc_config,
                config.genesis,
                genesis_block.header().hash(),
                client_actor.clone(),
                view_client.clone(),
            ),
        ));
    }}
");
    #[cfg(feature = "rosetta_rpc")]
    if let Some(rosetta_rpc_config) = config.rosetta_rpc_config {
        rpc_servers.push((
            "Rosetta RPC",
            near_rosetta_rpc::start_rosetta_rpc(
                rosetta_rpc_config,
                config.genesis,
                genesis_block.header().hash(),
                client_actor.clone(),
                view_client.clone(),
            ),
        ));
    }
    println!("--------------------------------------------------\n");


    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("rpc_servers.shrink_to_fit();");
    rpc_servers.shrink_to_fit();
    println!("--------------------------------------------------\n");

    println!("    tracing::trace!(target: \"diagnostic\", key = \"log\", \"Starting NEAR node with diagnostic activated\");\"");
    tracing::trace!(target: "diagnostic", key = "log", "Starting NEAR node with diagnostic activated");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("    let mut arbiters = vec![client_arbiter_handle, shards_manager_arbiter_handle];");
    let mut arbiters = vec![client_arbiter_handle, shards_manager_arbiter_handle];
    println!("--------------------------------------------------\n");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("    if let Some(db_metrics_arbiter) = db_metrics_arbiter {{");
    if let Some(db_metrics_arbiter) = db_metrics_arbiter {
        println!("        arbiters.push(db_metrics_arbiter);");
        arbiters.push(db_metrics_arbiter);
    }
    println!("    }}
\");");

    println!("\n--------------------------------------------------");
    print_file_path_and_function_name!();
    println!("      중요한");
    println!("Ok(NearNode {{
        client: client_actor,
        view_client,
        rpc_servers,
        arbiters,
        cold_store_loop_handle,
        state_sync_dump_handle,
    }})");
    Ok(NearNode {
        client: client_actor,
        view_client,
        rpc_servers,
        arbiters,
        cold_store_loop_handle,
        state_sync_dump_handle,
    })
}


pub struct RecompressOpts {
    pub dest_dir: PathBuf,
    pub keep_partial_chunks: bool,
    pub keep_invalid_chunks: bool,
    pub keep_trie_changes: bool,
}

pub fn recompress_storage(home_dir: &Path, opts: RecompressOpts) -> anyhow::Result<()> {
    print_file_path_and_function_name!();

    use strum::IntoEnumIterator;

    let config_path = home_dir.join(config::CONFIG_FILENAME);
    let config = config::Config::from_file(&config_path)
        .map_err(|err| anyhow::anyhow!("{}: {}", config_path.display(), err))?;
    let archive = config.archive;
    let mut skip_columns = Vec::new();
    if archive && !opts.keep_partial_chunks {
        skip_columns.push(DBCol::PartialChunks);
    }
    if archive && !opts.keep_invalid_chunks {
        skip_columns.push(DBCol::InvalidChunks);
    }
    if archive && !opts.keep_trie_changes {
        skip_columns.push(DBCol::TrieChanges);
    }

    let src_opener = NodeStorage::opener(home_dir, archive, &config.store, None);
    let src_path = src_opener.path();

    let mut dst_config = config.store.clone();
    dst_config.path = Some(opts.dest_dir);
    // Note: opts.dest_dir is resolved relative to current working directory
    // (since it’s a command line option) which is why we set home to cwd.
    let cwd = std::env::current_dir()?;
    let dst_opener = NodeStorage::opener(&cwd, archive, &dst_config, None);
    let dst_path = dst_opener.path();

    info!(target: "recompress",
          src = %src_path.display(), dest = %dst_path.display(),
          "Recompressing database");

    info!("Opening database at {}", src_path.display());
    let src_store = src_opener.open_in_mode(Mode::ReadOnly)?.get_hot_store();

    let final_head_height = if skip_columns.contains(&DBCol::PartialChunks) {
        let tip: Option<near_primitives::block::Tip> =
            src_store.get_ser(DBCol::BlockMisc, near_store::FINAL_HEAD_KEY)?;
        anyhow::ensure!(
            tip.is_some(),
            "{}: missing {}; is this a freshly set up node? note that recompress_storage makes no sense on those",
            src_path.display(),
            std::str::from_utf8(near_store::FINAL_HEAD_KEY).unwrap(),
        );
        tip.map(|tip| tip.height)
    } else {
        None
    };

    info!("Creating database at {}", dst_path.display());
    let dst_store = dst_opener.open_in_mode(Mode::Create)?.get_hot_store();

    const BATCH_SIZE_BYTES: u64 = 150_000_000;

    for column in DBCol::iter() {
        let skip = skip_columns.contains(&column);
        info!(
            target: "recompress",
            column_id = column as usize,
            %column,
            "{}",
            if skip { "Clearing  " } else { "Processing" }
        );
        if skip {
            continue;
        }

        let mut store_update = dst_store.store_update();
        let mut total_written: u64 = 0;
        let mut batch_written: u64 = 0;
        let mut count_keys: u64 = 0;
        for item in src_store.iter_raw_bytes(column) {
            let (key, value) = item.with_context(|| format!("scanning column {column}"))?;
            store_update.set_raw_bytes(column, &key, &value);
            total_written += value.len() as u64;
            batch_written += value.len() as u64;
            count_keys += 1;
            if batch_written >= BATCH_SIZE_BYTES {
                store_update.commit()?;
                info!(
                    target: "recompress",
                    column_id = column as usize,
                    %count_keys,
                    %total_written,
                    "Processing",
                );
                batch_written = 0;
                store_update = dst_store.store_update();
            }
        }
        info!(
            target: "recompress",
            column_id = column as usize,
            %count_keys,
            %total_written,
            "Done with "
        );
        store_update.commit()?;
    }

    // If we’re not keeping DBCol::PartialChunks, update chunk tail to point to
    // current final block.  If we don’t do that, the gc will try to work its
    // way from the genesis even though chunks at those heights have been
    // deleted.
    if skip_columns.contains(&DBCol::PartialChunks) {
        let chunk_tail = final_head_height.unwrap();
        info!(target: "recompress", %chunk_tail, "Setting chunk tail");
        let mut store_update = dst_store.store_update();
        store_update.set_ser(DBCol::BlockMisc, near_store::CHUNK_TAIL_KEY, &chunk_tail)?;
        store_update.commit()?;
    }

    core::mem::drop(dst_store);
    core::mem::drop(src_store);

    info!(target: "recompress", dest = %dst_path.display(), "Database recompressed");
    println!("\n------------ start_with_config_and_synchronization() 끝 ---------------------------");
    print_file_path_and_function_name!();

    println!("--------------------------------------------------\n");
    Ok(())
}
