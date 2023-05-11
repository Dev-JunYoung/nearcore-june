use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


#[cfg(unix)]
use anyhow::Context;
use near_amend_genesis::AmendGenesisCommand;
use near_chain_configs::GenesisValidationMode;
use near_client::ConfigUpdater;
use near_cold_store_tool::ColdStoreCommand;
use near_dyn_configs::{UpdateableConfigLoader, UpdateableConfigLoaderError, UpdateableConfigs};
use near_flat_storage::commands::FlatStorageCommand;
use near_jsonrpc_primitives::types::light_client::RpcLightClientExecutionProofResponse;
use near_mirror::MirrorCommand;
use near_network::tcp;
use near_o11y::tracing_subscriber::EnvFilter;
use near_o11y::{
    default_subscriber, default_subscriber_with_opentelemetry, BuildEnvFilterError,
    EnvFilterBuilder,
};
use near_ping::PingCommand;
use near_primitives::hash::CryptoHash;
use near_primitives::merkle::compute_root_from_path;
use near_primitives::types::{Gas, NumSeats, NumShards};
use near_state_parts::cli::StatePartsCommand;
use near_state_viewer::StateViewerSubCommand;
use near_store::db::RocksDB;
use near_store::Mode;
use near_undo_block::cli::UndoBlockCommand;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use tracing::{debug, error, info, warn};

/// NEAR Protocol Node
#[derive(clap::Parser)]
#[clap(version = crate::NEARD_VERSION_STRING.as_str())]
#[clap(subcommand_required = true, arg_required_else_help = true)]
pub(super) struct NeardCmd {
    #[clap(flatten)]
    opts: NeardOpts,
    #[clap(subcommand)]
    subcmd: NeardSubCommand,
}

impl NeardCmd {
    /// @func       parse_and_run()
    ///     @param      -
    ///     @summary    NeardCmd를 분석하고 실행함
    /// 특징
    ///     * neard_cmd
    ///     * NeardSubCommand
    ///     * home_dir
    ///     * genesis_validation
    ///     * neard_cmd
    ///
    /// genesis_validation
    ///
    pub(super) fn parse_and_run() -> anyhow::Result<()> {
        print_file_path_and_function_name!();

        let neard_cmd: Self = clap::Parser::parse();
        println!("\n--------------------------------------------------");
        println!("neard_cmd.opts :  {:#?}", neard_cmd.opts);
        println!("--------------------------------------------------\n");

        // Enable logging of the current thread.
        let _subscriber_guard = default_subscriber(
            make_env_filter(neard_cmd.opts.verbose_target())?,
            &neard_cmd.opts.o11y,
        )
            .local();

        info!(
            target: "neard",
            version = crate::NEARD_VERSION,
            build = crate::NEARD_BUILD,
            latest_protocol = near_primitives::version::PROTOCOL_VERSION
        );

        #[cfg(feature = "test_features")]
        {
            error!("THIS IS A NODE COMPILED WITH ADVERSARIAL BEHAVIORS. DO NOT USE IN PRODUCTION.");
            if std::env::var("ADVERSARY_CONSENT").unwrap_or_default() != "1" {
                error!(
                    "To run a node with adversarial behavior enabled give your consent \
                            by setting an environment variable:"
                );
                error!("ADVERSARY_CONSENT=1");
                std::process::exit(1);
            }
        }

        // println!("let home_dir = neard_cmd.opts.home.clone();"); // 출력 결과 : 없음
        let home_dir = neard_cmd.opts.home.clone();
        // println!("home_dir : {}",home_dir);

        println!("let genesis_validation = if neard_cmd.opts.unsafe_fast_startup {{
            GenesisValidationMode::UnsafeFast
        }} else {{
            GenesisValidationMode::Full
        }};");
        let genesis_validation = if neard_cmd.opts.unsafe_fast_startup {
            GenesisValidationMode::UnsafeFast
        } else {
            GenesisValidationMode::Full
        };
        // println!("\n--------------------------------------------------");
        // print_file_path_and_function_name!();
        // println!("      --------------------------------------------------");
        // println!("  home_dir :  {:#?}", home_dir);
        // println!("  genesis_validation :  {:#?},{}", genesis_validation, "의미 : 안전하진 않지만 빠르게 제네시스 검증함");
        // println!("--------------------------------------------------\n");

        println!("match neard_cmd.subcmd");
        match neard_cmd.subcmd {
            NeardSubCommand::Init(cmd) => cmd.run(&home_dir)?,
            NeardSubCommand::Localnet(cmd) => cmd.run(&home_dir),
            NeardSubCommand::Run(cmd) => cmd.run(
                &home_dir,
                genesis_validation,
                neard_cmd.opts.verbose_target(),
                &neard_cmd.opts.o11y,
            ),
            NeardSubCommand::StateViewer(cmd) => {
                print!("line: {},{:?} ",std::line!(), std::thread::current().id());
                println!("let mode = if cmd.readwrite {{Mode::ReadWrite}} else {{Mode::ReadOnly }};");
                let mode = if cmd.readwrite { Mode::ReadWrite } else { Mode::ReadOnly };
                print!("line: {},{:?} ",std::line!(), std::thread::current().id());
                println!("cmd.subcmd.run(&home_dir, genesis_validation, mode, cmd.store_temperature);");
                cmd.subcmd.run(&home_dir, genesis_validation, mode, cmd.store_temperature);
            }
            NeardSubCommand::RecompressStorage(cmd) => {
                print!("line: {},{:?} ",std::line!(), std::thread::current().id());
                println!("NeardSubCommand::RecompressStorage(cmd) => {{
                cmd.run(&home_dir);
            }}");
                cmd.run(&home_dir);
            }
            NeardSubCommand::VerifyProof(cmd) => {
                cmd.run();
                print!("line: {},{:?} ",std::line!(), std::thread::current().id());
                println!("NeardSubCommand::VerifyProof(cmd) => {{
                cmd.run(&home_dir);
            }}");
            }
            NeardSubCommand::Ping(cmd) => {
                cmd.run()?;
                print!("line: {},{:?} ",std::line!(), std::thread::current().id());
                println!("NeardSubCommand::Ping(cmd) => {{
                cmd.run(&home_dir);
            }}");
            }
            NeardSubCommand::Mirror(cmd) => {
                cmd.run()?;
                print!("line: {},{:?} ",std::line!(), std::thread::current().id());
                println!("NeardSubCommand::Mirror(cmd) => {{
                cmd.run(&home_dir);
            }}");
            }
            NeardSubCommand::AmendGenesis(cmd) => {
                cmd.run()?;
                print!("line: {},{:?} ",std::line!(), std::thread::current().id());
                println!("NeardSubCommand::AmendGenesis(cmd) => {{
                cmd.run(&home_dir);
            }}");
            }
            NeardSubCommand::ColdStore(cmd) => {
                cmd.run(&home_dir)?;
                print!("line: {},{:?} ",std::line!(), std::thread::current().id());
                println!("NeardSubCommand::ColdStore(cmd) => {{
                cmd.run(&home_dir);
            }}");
            }
            NeardSubCommand::StateParts(cmd) => {
                cmd.run()?;
                print!("line: {},{:?} ",std::line!(), std::thread::current().id());
                println!("NeardSubCommand::StateParts(cmd) => {{
                cmd.run(&home_dir);
            }}");
            }
            NeardSubCommand::FlatStorage(cmd) => {
                cmd.run(&home_dir)?;
                print!("line: {},{:?} ",std::line!(), std::thread::current().id());
                println!("NeardSubCommand::FlatStorage(cmd) => {{
                cmd.run(&home_dir);
            }}");
            }
            NeardSubCommand::ValidateConfig(cmd) => {
                cmd.run(&home_dir)?;
                print!("line: {},{:?} ",std::line!(), std::thread::current().id());
                println!("NeardSubCommand::ValidateConfig(cmd) => {{
                cmd.run(&home_dir);
            }}");
            }
            NeardSubCommand::UndoBlock(cmd) => {
                cmd.run(&home_dir, genesis_validation)?;
                print!("line: {},{:?} ",std::line!(), std::thread::current().id());
                println!("NeardSubCommand::UndoBlock(cmd) => {{
                cmd.run(&home_dir);
            }}");
            }
        };
        Ok(())
    }
}

#[derive(clap::Parser)]
pub(super) struct StateViewerCommand {
    /// By default state viewer opens rocks DB in the read only mode, which allows it to run
    /// multiple instances in parallel and be sure that no unintended changes get written to the DB.
    /// In case an operation needs to write to caches, a read-write mode may be needed.
    #[clap(long, short = 'w')]
    readwrite: bool,
    /// What store temperature should the state viewer open. Allowed values are hot and cold but
    /// cold is only available when cold_store is configured.
    /// Cold temperature actually means the split store will be used.
    #[clap(long, short = 't', default_value = "hot")]
    store_temperature: near_store::Temperature,
    #[clap(subcommand)]
    subcmd: StateViewerSubCommand,
}

#[derive(clap::Parser, Debug)]
struct NeardOpts {
    /// Sets verbose logging for the given target, or for all targets if no
    /// target is given.
    #[clap(long, name = "target")]
    verbose: Option<Option<String>>,
    /// Directory for config and data.
    #[clap(long, parse(from_os_str), default_value_os = crate::DEFAULT_HOME.as_os_str())]
    home: PathBuf,
    /// Skips consistency checks of genesis.json (and records.json) upon startup.
    /// Let's you start `neard` slightly faster.
    #[clap(long)]
    unsafe_fast_startup: bool,
    /// Enables export of span data using opentelemetry protocol.
    #[clap(flatten)]
    o11y: near_o11y::Options,
}

impl NeardOpts {
    pub fn verbose_target(&self) -> Option<&str> {
        // print_file_path_and_function_name!();
        match self.verbose {
            None => None,
            Some(None) => Some(""),
            Some(Some(ref target)) => Some(target.as_str()),
        }
    }
}

#[derive(clap::Parser)]
pub(super) enum NeardSubCommand {
    /// Initializes NEAR configuration
    Init(InitCmd),
    /// Runs NEAR node
    Run(RunCmd),
    /// Sets up local configuration with all necessary files (validator key, node key, genesis and
    /// config)
    Localnet(LocalnetCmd),
    /// View DB state.
    #[clap(name = "view-state", alias = "view_state")]
    StateViewer(StateViewerCommand),
    /// Recompresses the entire storage.  This is a slow operation which reads
    /// all the data from the database and writes them down to a new copy of the
    /// database.
    ///
    /// In 1.26 release the compression algorithm for the database has changed
    /// to reduce storage size.  Nodes don’t need to do anything for new data to
    /// take advantage of better compression but existing data may take months
    /// to be recompressed.  This may be an issue for archival nodes which keep
    /// hold of all the old data.
    ///
    /// This command makes it possible to force the recompression as a one-time
    /// operation.  Using it reduces the database even by up to 40% though that
    /// is partially due to database ‘defragmentation’ (whose effects will wear
    /// off in time).  Still, reduction by about 20% even if that’s taken into
    /// account can be expected.
    ///
    /// It’s important to remember however, that this command may take up to
    /// a day to finish in which time the database cannot be used by the node.
    ///
    /// Furthermore, file system where output directory is located needs enough
    /// free space to store the new copy of the database.  It will be smaller
    /// than the original but to be safe one should provision around the same
    /// space as the size of the current `data` directory.
    ///
    /// Finally, because this command is meant only as a temporary migration
    /// tool, it is planned to be removed by the end of 2022.
    #[clap(alias = "recompress_storage")]
    RecompressStorage(RecompressStorageSubCommand),

    /// Verify proofs
    #[clap(alias = "verify_proof")]
    VerifyProof(VerifyProofSubCommand),

    /// Connects to a NEAR node and sends ping messages to the accounts it sends
    /// us after the handshake is completed, printing stats to stdout.
    Ping(PingCommand),

    /// Mirror transactions from a source chain to a test chain with state forked
    /// from it, reproducing traffic and state as closely as possible.
    Mirror(MirrorCommand),

    /// Amend a genesis/records file created by `dump-state`.
    AmendGenesis(AmendGenesisCommand),

    /// Testing tool for cold storage
    ColdStore(ColdStoreCommand),

    /// Connects to a NEAR node and sends state parts requests after the handshake is completed.
    StateParts(StatePartsCommand),

    /// Flat storage related tooling.
    FlatStorage(FlatStorageCommand),

    /// validate config files including genesis.json and config.json
    ValidateConfig(ValidateConfigCommand),

    /// reset the head of the chain locally to the prev block of current head
    UndoBlock(UndoBlockCommand),
}

#[derive(clap::Parser)]
pub(super) struct InitCmd {
    /// Download the verified NEAR genesis file automatically.
    #[clap(long)]
    download_genesis: bool,
    /// Download the verified NEAR config file automatically.
    #[clap(long)]
    download_config: bool,
    /// Makes block production fast (TESTING ONLY).
    #[clap(long)]
    fast: bool,
    /// Account ID for the validator key.
    #[clap(long)]
    account_id: Option<String>,
    /// Chain ID, by default creates new random.
    #[clap(long, forbid_empty_values = true)]
    chain_id: Option<String>,
    /// Specify a custom download URL for the genesis file.
    #[clap(long)]
    download_genesis_url: Option<String>,
    /// Specify a custom download URL for the records file.
    #[clap(long)]
    download_records_url: Option<String>,
    /// Specify a custom download URL for the config file.
    #[clap(long)]
    download_config_url: Option<String>,
    /// Genesis file to use when initializing testnet (including downloading).
    #[clap(long)]
    genesis: Option<String>,
    /// Initialize boots nodes in <node_key>@<ip_addr> format seperated by commas
    /// to bootstrap the network and store them in config.json
    #[clap(long)]
    boot_nodes: Option<String>,
    /// Number of shards to initialize the chain with.
    #[clap(long, default_value = "1")]
    num_shards: NumShards,
    /// Specify private key generated from seed (TESTING ONLY).
    #[clap(long)]
    test_seed: Option<String>,
    /// Customize max_gas_burnt_view runtime limit.  If not specified, value
    /// from genesis configuration will be taken.
    #[clap(long)]
    max_gas_burnt_view: Option<Gas>,
}

/// Warns if unsupported build of the executable is used on mainnet or testnet.
///
/// Verifies that when running on mainnet or testnet chain a neard binary built
/// with `make release` command is used.  That Makefile targets enable
/// optimisation options which aren’t enabled when building with different
/// methods and is the only officially supported method of building the binary
/// to run in production.
///
/// The detection is done by checking that `NEAR_RELEASE_BUILD` environment
/// variable was set to `release` during compilation (which is what Makefile
/// sets) and that neither `nightly` nor `nightly_protocol` features are
/// enabled.
fn check_release_build(chain: &str) {
    print_file_path_and_function_name!();

    let is_release_build = option_env!("NEAR_RELEASE_BUILD") == Some("release")
        && !cfg!(feature = "nightly")
        && !cfg!(feature = "nightly_protocol");
    if !is_release_build && ["mainnet", "testnet"].contains(&chain) {
        warn!(
            target: "neard",
            "Running a neard executable which wasn’t built with `make release` \
             command isn’t supported on {}.",
            chain
        );
        warn!(
            target: "neard",
            "Note that `cargo build --release` builds lack optimisations which \
             may be needed to run properly on {}",
            chain
        );
        warn!(
            target: "neard",
            "Consider recompiling the binary using `make release` command.");
    }
}

impl InitCmd {
    pub(super) fn run(self, home_dir: &Path) -> anyhow::Result<()> {
        print_file_path_and_function_name!();

        // TODO: Check if `home` exists. If exists check what networks we already have there.
        if (self.download_genesis || self.download_genesis_url.is_some()) && self.genesis.is_some()
        {
            anyhow::bail!("Please give either --genesis or --download-genesis, not both.");
        }

        if let Some(chain) = self.chain_id.as_ref() {
            check_release_build(chain)
        }

        nearcore::init_configs(
            home_dir,
            self.chain_id,
            self.account_id.and_then(|account_id| account_id.parse().ok()),
            self.test_seed.as_deref(),
            self.num_shards,
            self.fast,
            self.genesis.as_deref(),
            self.download_genesis,
            self.download_genesis_url.as_deref(),
            self.download_records_url.as_deref(),
            self.download_config,
            self.download_config_url.as_deref(),
            self.boot_nodes.as_deref(),
            self.max_gas_burnt_view,
        )
            .context("Failed to initialize configs")
    }
}

#[derive(clap::Parser)]
pub(super) struct RunCmd {
    /// Configure node to run as archival node which prevents deletion of old
    /// blocks.  This is a persistent setting; once client is started as
    /// archival node, it cannot be run in non-archival mode.
    #[clap(long)]
    archive: bool,
    /// Set the boot nodes to bootstrap network from.
    #[clap(long)]
    boot_nodes: Option<String>,
    /// Whether to re-establish connections from the ConnectionStore on startup
    #[clap(long)]
    connect_to_reliable_peers_on_startup: Option<bool>,
    /// Minimum number of peers to start syncing/producing blocks
    #[clap(long)]
    min_peers: Option<usize>,
    /// Customize network listening address (useful for running multiple nodes on the same machine).
    #[clap(long)]
    network_addr: Option<SocketAddr>,
    /// Set this to false to only produce blocks when there are txs or receipts (default true).
    #[clap(long)]
    produce_empty_blocks: Option<bool>,
    /// Customize RPC listening address (useful for running multiple nodes on
    /// the same machine).  Ignored if ‘--disable-rpc’ is given.
    #[cfg(feature = "json_rpc")]
    #[clap(long)]
    rpc_addr: Option<String>,
    /// Export prometheus metrics on an additional listening address, which is useful
    /// for having separate access restrictions for the RPC and prometheus endpoints.
    /// Ignored if RPC http server is disabled, see 'rpc_addr'.
    #[cfg(feature = "json_rpc")]
    #[clap(long)]
    rpc_prometheus_addr: Option<String>,
    /// Disable the RPC endpoint.  This is a no-op on builds which don’t support
    /// RPC endpoint.
    #[clap(long)]
    #[allow(dead_code)]
    disable_rpc: bool,
    /// Customize telemetry url.
    #[clap(long)]
    telemetry_url: Option<String>,
    /// Customize max_gas_burnt_view runtime limit.  If not specified, either
    /// value given at ‘init’ (i.e. present in config.json) or one from genesis
    /// configuration will be taken.
    #[clap(long)]
    max_gas_burnt_view: Option<Gas>,
}

impl RunCmd {
    pub(super) fn run(
        self,
        home_dir: &Path,
        genesis_validation: GenesisValidationMode,
        verbose_target: Option<&str>,
        o11y_opts: &near_o11y::Options,
    ) {
        print_file_path_and_function_name!();
        println!("let mut near_config = nearcore::config::load_config()");
        // Load configs from home.
        println!("      중요! = NEAR에 대한 전반적인 config 로딩 시작");
        let mut near_config = nearcore::config::load_config(home_dir, genesis_validation)
            .unwrap_or_else(|e| panic!("Error loading config: {:#}", e));
        println!("--------------------------------------------------\n");

        println!("\n--------------------------------------------------");
        // println!("      --------------------------------------------------");
        print_file_path_and_function_name!();
        println!("home_dir :  {:#?}", home_dir);
        // println!("      --------------------------------------------------");
        print_file_path_and_function_name!();
        println!("  중요! = 이것 저것 왕창 찍힘~ !!");
        println!("near_config.config :  {:#?}", near_config.config);
        println!("챗지피티 해석
   1. `genesis_file`: 체인의 초기 상태를 정의하는 JSON 파일의 경로입니다.
   2. `genesis_records_file`: 초기 레코드를 포함하는 JSON 파일의 경로입니다. 기본적으로 `genesis_file`에서 생성됩니다.
   3. `validator_key_file`: 노드가 검증자로서 작동하기 위해 사용되는 키 파일의 경로입니다.
   4. `node_key_file`: 노드 간 P2P 통신에 사용되는 키 파일의 경로입니다.
   5. `rpc`: JSON-RPC 인터페이스 설정을 포함하는 섹션입니다.
      - `addr`: JSON-RPC 서버가 수신 대기할 IP 주소 및 포트입니다.
      - `prometheus_addr`: Prometheus 서버에 대한 설정입니다.
      - `cors_allowed_origins`: CORS 요청을 허용할 도메인 목록입니다.
      - `polling_config`: RPC 요청 폴링에 대한 설정입니다.
      - `limits_config`: 요청 크기 및 처리 제한을 포함합니다.
      - `enable_debug_rpc`: 디버그 RPC를 활성화할지 여부를 나타냅니다.
      - `experimental_debug_pages_src_path`: 디버그 페이지의 소스 경로입니다.
   6. `rosetta_rpc`: Rosetta API에 대한 설정입니다.
   7. `telemetry`: 노드 통계를 수집하고 보고하는 방법에 대한 설정입니다.
      - `endpoints`: 텔레메트리 데이터를 전송할 엔드포인트입니다.
      - `reporting_interval`: 보고 주기를 설정합니다.
   8. `network`: P2P 네트워크 설정입니다.
      - `addr`: 노드가 수신 대기할 IP 주소 및 포트입니다.
      - `boot_nodes`: 부트스트랩 노드 목록입니다.
      - `whitelist_nodes`: 화이트리스트에 있는 노드와만 연결합니다.
      - `max_num_peers`: 최대 피어 수입니다.
      - `minimum_outbound_peers`: 최소 아웃바운드 피어 수입니다.
      - `ideal_connections_lo`: 이상적인 최소 연결 수입니다.
      - `ideal_connections_hi`: 이상적인 최대 연결 수입니다.
      - `peer_recent_time_window`: 최근 피어로 간주되는 시간 창입니다.
      - `safe_set_size`: 안전한 피어 세트 크기입니다.
      - `archival_peer_connections_lower_bound`: 아카이브 노드와의 최소 연결 수입니다.
      - `handshake_timeout`: 핸드셰이크 시간 초과입니다.
      - `skip_sync_wait`: 동기화 대기를 건너뛸지 여부입니다.
      - `ban_window`: 피어가 차단되는 시간입니다.
      - `blacklist`: 차단된 피어 목록입니다.
      - `ttl_account_id_router`: 계정 ID

      - `ttl_account_id_router`: 계정 ID 라우터의 TTL(시간 제한)입니다.
      - `peer_stats_period`: 피어 통계를 수집하는 주기입니다.
      - `monitor_peers_max_period`: 피어 모니터링의 최대 주기입니다.
      - `peer_states_cache_size`: 피어 상태 캐시 크기입니다.
      - `peer_expiration_duration`: 피어 만료 기간입니다.
      - `public_addrs`: 공개 주소 목록입니다.
      - `allow_private_ip_in_public_addrs`: 공개 주소 목록에 사설 IP를 허용할지 여부입니다.
      - `trusted_stun_servers`: 신뢰할 수 있는 STUN 서버 목록입니다.
      - `experimental`: 실험적 네트워크 기능 설정입니다.
   9. `consensus`: 합의 알고리즘에 관련된 설정입니다.
      - `min_num_peers`: 최소 피어 수입니다.
      - `block_production_tracking_delay`: 블록 생성 추적 지연입니다.
      - `min_block_production_delay`: 최소 블록 생성 지연입니다.
      - `max_block_production_delay`: 최대 블록 생성 지연입니다.
      - `max_block_wait_delay`: 최대 블록 대기 지연입니다.
      - `produce_empty_blocks`: 빈 블록 생성을 활성화할지 여부입니다.
      - `block_fetch_horizon`: 블록을 가져올 수 있는 최대 높이입니다.
      - `state_fetch_horizon`: 상태를 가져올 수 있는 최대 높이입니다.
      - `block_header_fetch_horizon`: 블록 헤더를 가져올 수 있는 최대 높이입니다.
      - `catchup_step_period`: 동기화 단계 주기입니다.
      - `chunk_request_retry_period`: 청크 요청 재시도 주기입니다.
      - `header_sync_initial_timeout`: 초기 헤더 동기화 시간 초과입니다.
      - `header_sync_progress_timeout`: 진행 중인 헤더 동기화 시간 초과입니다.
      - `header_sync_stall_ban_timeout`: 정체된 헤더 동기화 차단 시간 초과입니다.
      - `state_sync_timeout`: 상태 동기화 시간 초과입니다.
      - `header_sync_expected_height_per_second`: 초당 예상 헤더 동기화 높이입니다.
      - `sync_check_period`: 동기화 확인 주기입니다.
      - `sync_step_period`: 동기화 단계 주기입니다.
      - `doomslug_step_period`: Doomslug 단계 주기입니다.
      - `sync_height_threshold`: 동기화 높이 임계값입니다.
   10. `tracked_accounts`: 추적할 계정의 목록입니다.
   11. `tracked_shards`: 추적할 샤드의 목록입니다.
   12. `tracked_shard_schedule`: 추적할 샤드 스케줄입니다.
   13. `archive`: 아카이브 : 아카이브 노드 설정.

      - `stored_roots`: 아카이브 노드에 저장될 루트 수.
      - `pruning_horizon`: 노드가 데이터를 정리한 후 블록 수.

   14. `store` 저장소: 저장소 설정.

      - `db_path`: 데이터베이스 디렉토리 경로.
      - `gc_blocks_limit`: 가비지 수집을 시작하기 전에 처리해야 하는 블록 수입니다.
      - `gc_time_limit`: 가비지 수집에 허용되는 최대 시간입니다.
      - `gc_period`: 가비지 수집 실행 사이의 시간 간격입니다.
      - `storage_size_limit`: 노드에 허용되는 최대 저장소 크기입니다.
      - `ledger_depth`: 원장에 사용되는 Merkle 트리의 깊이.
      - `trie_depth`: state trie에 사용되는 Merkle 트리의 깊이입니다.
      - `cache_size`: 데이터베이스 작업에 사용되는 캐시 크기입니다.
      - `num_threads`: 데이터베이스 작업에 사용되는 스레드 수입니다.
      - `flush_every`: 데이터베이스 플러시가 트리거되기 전의 작업 수입니다.

   15. `transaction_pool`: 트랜잭션 풀 설정.

      -`max_count`: 풀에서 허용되는 최대 트랜잭션 수입니다.
      -`max_memory_usage`: 트랜잭션 풀에 허용되는 최대 메모리 사용량.
      -`eviction_timeout`: 트랜잭션이 풀에서 제거되는 시간입니다.
      -`resubmission_period`: 트랜잭션이 풀에 다시 제출되기까지의 시간입니다.
      -`max_timestamp_diff`: 트랜잭션과 현재 시간 사이의 타임스탬프 최대 차이.

   16. `logging`: 로깅 설정.

      -`log_level`: 로그 수준(예: \"오류\", \"경고\", \"정보\", \"디버그\", \"추적\").
      -`log_target`: 로그 출력 대상(예: \"stdout\", \"stderr\", \"file\").
      -`log_file_path`: 로그 파일 경로(로그 대상으로 \"file\"이 선택된 경우).
      -`log_max_file_size`: 최대 로그 파일 크기.
      -`log_max_backup_count`: 보관할 백업 로그 파일의 최대 개수입니다.
      -`log_max_age`: 로그 파일이 제거되기 전의 최대 로그 파일 수명입니다.
      -`log_rotation_period`: 로그 파일 순환을 위한 시간 간격.
");
//         println!("near_config.config :  {{
//     genesis_file: 'genesis.json',
//     genesis_records_file: None,
//     validator_key_file: 'validator_key.json',
//     node_key_file: 'node_key.json',
//     rpc:
//     telemetry: ~
//     network: ~,
//     consensus: ~,
//     tracked_accounts: [],
//     tracked_shards: [],
// ..
// }}");
// println!("      --------------------------------------------------");
        // println!("          near_config.network_config :  {:#?}", near_config.client_config.config);
        // println!("          near_config.network_config :  {:#?}", near_config.client_config.client_config);
        // println!("          near_config.network_config :  {:#?}", near_config.client_config.network_config);
        // println!("          near_config.network_config :  {:#?}", near_config.client_config.rpc_config);
        // println!("          near_config.network_config :  {:#?}", near_config.client_config.rosetta_rpc_config);
        // println!("          near_config.network_config :  {:#?}", near_config.client_config.telemetry_config);
        // println!("          near_config.network_config :  {:#?}", near_config.client_config.genesis);
        // println!("          near_config.network_config :  {:#?}", near_config.client_config.validator_signer);

        // println!("      --------------------------------------------------");
        // print_file_path_and_function_name!();
        // println!("near_config.genesis :     {:#?}", near_config.genesis);
        // println!("      --------------------------------------------------");
        // print_file_path_and_function_name!();
        // println!("near_config.rpc_config :  {:#?}", near_config.rpc_config);
        // println!("      --------------------------------------------------");
        // print_file_path_and_function_name!();
        // println!("near_config.rosetta_rpc_config :  {:#?}", near_config.rosetta_rpc_config);
        // println!("      --------------------------------------------------");
        // print_file_path_and_function_name!();
        // println!("near_config.telemetry_config :    {:#?}", near_config.telemetry_config);
        println!("      텔레메트리(telemetry) : 원격측정");
        // println!("      텔레메트리(telemetry) : NEAR 프로토콜의 맥락에서 원격 측정은 NEAR 네트워크에서 실행되는 노드의 다양한 메트릭 및 데이터 포인트 수집을 의미합니다. 이 정보는 네트워크의 개발자, 검증자 및 기타 참여자가 네트워크의 성능, 안정성 및 상태를 모니터링하는 데 도움이 됩니다.");
        println!("--------------------------------------------------\n");

        // println!("          near_config.validator_signer :    {:#?}", near_config.validator_signer);
        check_release_build(&near_config.client_config.chain_id);

        // Set current version in client config.
        near_config.client_config.version = crate::neard_version();
        // Override some parameters from command line.
        if let Some(produce_empty_blocks) = self.produce_empty_blocks {
            near_config.client_config.produce_empty_blocks = produce_empty_blocks;
        }
        if let Some(connect_to_reliable_peers_on_startup) =
            self.connect_to_reliable_peers_on_startup
        {
            near_config.network_config.connect_to_reliable_peers_on_startup =
                connect_to_reliable_peers_on_startup;
        }
        if let Some(boot_nodes) = self.boot_nodes {
            if !boot_nodes.is_empty() {
                near_config.network_config.peer_store.boot_nodes = boot_nodes
                    .split(',')
                    .map(|chunk| chunk.parse().expect("Failed to parse PeerInfo"))
                    .collect();
            }
        }
        if let Some(min_peers) = self.min_peers {
            near_config.client_config.min_num_peers = min_peers;
        }
        if let Some(network_addr) = self.network_addr {
            near_config.network_config.node_addr =
                Some(near_network::tcp::ListenerAddr::new(network_addr));
        }
        #[cfg(feature = "json_rpc")]
        if self.disable_rpc {
            near_config.rpc_config = None;
        } else {
            if let Some(rpc_addr) = self.rpc_addr {
                near_config.rpc_config.get_or_insert(Default::default()).addr =
                    tcp::ListenerAddr::new(rpc_addr.parse().unwrap());
            }
            if let Some(rpc_prometheus_addr) = self.rpc_prometheus_addr {
                near_config.rpc_config.get_or_insert(Default::default()).prometheus_addr =
                    Some(rpc_prometheus_addr);
            }
        }
        if let Some(telemetry_url) = self.telemetry_url {
            if !telemetry_url.is_empty() {
                near_config.telemetry_config.endpoints.push(telemetry_url);
            }
        }
        if self.archive {
            near_config.client_config.archive = true;
        }
        if self.max_gas_burnt_view.is_some() {
            near_config.client_config.max_gas_burnt_view = self.max_gas_burnt_view;
        }

        #[cfg(feature = "sandbox")]
        {
            if near_config.client_config.chain_id == "mainnet"
                || near_config.client_config.chain_id == "testnet"
                || near_config.client_config.chain_id == "betanet"
            {
                eprintln!(
                    "Sandbox node can only run dedicate localnet, cannot connect to a network"
                );
                std::process::exit(1);
            }
        }

        let (tx_crash, mut rx_crash) = broadcast::channel::<()>(16);
        let (tx_config_update, rx_config_update) =
            broadcast::channel::<Result<UpdateableConfigs, Arc<UpdateableConfigLoaderError>>>(16);
        let sys = actix::System::new();
        println!("\n--------------------------------------------------");
        // println!("      --------------------------------------------------");
        print_file_path_and_function_name!();
        println!("tx_config_update :  {:#?}", tx_config_update);
        // println!("      --------------------------------------------------");
        print_file_path_and_function_name!();
        println!("rx_config_update :  {:#?}", rx_config_update);
        println!("--------------------------------------------------\n");


        println!("  중요  sys.block_on(async move{{}} ");
        print_file_path_and_function_name!();
        sys.block_on(async move {
            // Initialize the subscriber that takes care of both logging and tracing.
            let _subscriber_guard = default_subscriber_with_opentelemetry(
                make_env_filter(verbose_target).unwrap(),
                o11y_opts,
                near_config.client_config.chain_id.clone(),
                near_config.network_config.node_key.public_key().clone(),
                near_config
                    .network_config
                    .validator
                    .as_ref()
                    .map(|validator| validator.account_id()),
            )
                .await
                .global();

            let updateable_configs = nearcore::dyn_config::read_updateable_configs(home_dir)
                .unwrap_or_else(|e| panic!("Error reading dynamic configs: {:#}", e));
            // println!("      --------------------------------------------------");
            // println!("updateable_configs.client_config :  {:#?}", updateable_configs.client_config);
            println!("updateable_configs.log_config :  {:#?}", updateable_configs.log_config);
            // println!("      --------------------------------------------------");
            println!("--------------------------------------------------\n");

            let mut updateable_config_loader =
                UpdateableConfigLoader::new(updateable_configs.clone(), tx_config_update);
            let config_updater = ConfigUpdater::new(rx_config_update);

            let nearcore::NearNode {
                client,//
                view_client,//
                arbiters,//
                rpc_servers,//
                cold_store_loop_handle,//
                state_sync_dump_handle//
            } = nearcore::start_with_config_and_synchronization(
                home_dir,
                near_config,
                Some(tx_crash),
                Some(config_updater),
            )
                .expect("start_with_config");

            let sig = loop {
                let sig = wait_for_interrupt_signal(home_dir, &mut rx_crash).await;
                if sig == "SIGHUP" {
                    let maybe_updateable_configs =
                        nearcore::dyn_config::read_updateable_configs(home_dir);
                    updateable_config_loader.reload(maybe_updateable_configs);
                } else {
                    break sig;
                }
                println!("\n--------------------------------------------------");
                print_file_path_and_function_name!();
                // println!("      --------------------------------------------------");
                println!("sig :  {:#?}", sig);
            };

            println!("\n--------------------------------------------------");
            println!("  ----------------여기 특이점 : 로그가 안찍힘-------------------");
            println!("  ----------------이유 : 윗줄이 무한루프 도는 듯?-------------------");
            print_file_path_and_function_name!();
            // println!("      --------------------------------------------------");
            println!("sig :  {:#?}", sig);
            warn!(target: "neard", "{}, stopping... this may take a few minutes.", sig);

            if let Some(handle) = cold_store_loop_handle {
                handle.stop()
            }
            if let Some(handle) = state_sync_dump_handle {
                handle.stop()
            }
            futures::future::join_all(rpc_servers.iter().map(|(name, server)| async move {
                server.stop(true).await;
                debug!(target: "neard", "{} server stopped", name);
            }))
                .await;
            actix::System::current().stop();
            // Disable the subscriber to properly shutdown the tracer.
            near_o11y::reload(Some("error"), None, Some(near_o11y::OpenTelemetryLevel::OFF))
                .unwrap();
            println!("      --------------  sys.block_on(async move{{}}) 끝 ------------------------------------");
            println!("--------------------------------------------------\n");
        });
        sys.run().unwrap();
        info!(target: "neard", "Waiting for RocksDB to gracefully shutdown");
        RocksDB::block_until_all_instances_are_dropped();
    }
}

#[cfg(not(unix))]
async fn wait_for_interrupt_signal(_home_dir: &Path, mut _rx_crash: &Receiver<()>) -> &str {
    print_file_path_and_function_name!();

    // TODO(#6372): Support graceful shutdown on windows.
    tokio::signal::ctrl_c().await.unwrap();
    "Ctrl+C"
}

#[cfg(unix)]
async fn wait_for_interrupt_signal(_home_dir: &Path, rx_crash: &mut Receiver<()>) -> &'static str {
    print_file_path_and_function_name!();

    use tokio::signal::unix::{signal, SignalKind};
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let mut sighup = signal(SignalKind::hangup()).unwrap();

    tokio::select! {
         _ = sigint.recv()  => "SIGINT",
         _ = sigterm.recv() => "SIGTERM",
         _ = sighup.recv() => "SIGHUP",
         _ = rx_crash.recv() => "ClientActor died",
    }
}

#[derive(clap::Parser)]
pub(super) struct LocalnetCmd {
    /// Number of non-validators to initialize the localnet with.
    #[clap(short = 'n', long, alias = "n", default_value = "0")]
    non_validators: NumSeats,
    /// Prefix for the directory name for each node with (e.g. ‘node’ results in
    /// ‘node0’, ‘node1’, ...)
    #[clap(long, default_value = "node")]
    prefix: String,
    /// Number of shards to initialize the localnet with.
    #[clap(short = 's', long, default_value = "1")]
    shards: NumShards,
    /// Number of validators to initialize the localnet with.
    #[clap(short = 'v', long, alias = "v", default_value = "4")]
    validators: NumSeats,
    /// Whether to create fixed shards accounts (that are tied to a given
    /// shard).
    #[clap(long)]
    fixed_shards: bool,
    /// Whether to configure nodes as archival.
    #[clap(long)]
    archival_nodes: bool,
    /// Comma separated list of shards to track, the word 'all' to track all shards or the word 'none' to track no shards.
    #[clap(long, default_value = "all")]
    tracked_shards: String,
}

impl LocalnetCmd {
    fn parse_tracked_shards(tracked_shards: &str, num_shards: NumShards) -> Vec<u64> {
        print_file_path_and_function_name!();

        if tracked_shards.to_lowercase() == "all" {
            return (0..num_shards).collect();
        }
        if tracked_shards.to_lowercase() == "none" {
            return vec![];
        }
        tracked_shards
            .split(',')
            .map(|shard_id| shard_id.parse::<u64>().expect("Shard id must be an integer"))
            .collect()
    }

    pub(super) fn run(self, home_dir: &Path) {
        print_file_path_and_function_name!();

        let tracked_shards = Self::parse_tracked_shards(&self.tracked_shards, self.shards);

        nearcore::config::init_testnet_configs(
            home_dir,
            self.shards,
            self.validators,
            self.non_validators,
            &self.prefix,
            true,
            self.archival_nodes,
            self.fixed_shards,
            tracked_shards,
        );
    }
}

#[derive(clap::Args)]
#[clap(arg_required_else_help = true)]
pub(super) struct RecompressStorageSubCommand {
    /// Directory where to save new storage.
    #[clap(long)]
    output_dir: PathBuf,

    /// Keep data in DBCol::PartialChunks column.  Data in that column can be
    /// reconstructed from DBCol::Chunks is not needed by archival nodes.  This is
    /// always true if node is not an archival node.
    #[clap(long)]
    keep_partial_chunks: bool,

    /// Keep data in DBCol::InvalidChunks column.  Data in that column is only used
    /// when receiving chunks and is not needed to serve archival requests.
    /// This is always true if node is not an archival node.
    #[clap(long)]
    keep_invalid_chunks: bool,

    /// Keep data in DBCol::TrieChanges column.  Data in that column is never used
    /// by archival nodes.  This is always true if node is not an archival node.
    #[clap(long)]
    keep_trie_changes: bool,
}

impl RecompressStorageSubCommand {
    pub(super) fn run(self, home_dir: &Path) {
        print_file_path_and_function_name!();

        warn!(target: "neard", "Recompressing storage; note that this operation may take up to a day to finish.");
        let opts = nearcore::RecompressOpts {
            dest_dir: self.output_dir,
            keep_partial_chunks: self.keep_partial_chunks,
            keep_invalid_chunks: self.keep_invalid_chunks,
            keep_trie_changes: self.keep_trie_changes,
        };
        if let Err(err) = nearcore::recompress_storage(home_dir, opts) {
            error!("{}", err);
            std::process::exit(1);
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum VerifyProofError {
    #[error("invalid outcome root proof")]
    InvalidOutcomeRootProof,
    #[error("invalid block hash proof")]
    InvalidBlockHashProof,
}

#[derive(clap::Parser)]
pub struct VerifyProofSubCommand {
    #[clap(long)]
    json_file_path: String,
}

impl VerifyProofSubCommand {
    /// Verifies light client transaction proof (result of the EXPERIMENTAL_light_client_proof RPC call).
    /// Returns the Hash and height of the block that transaction belongs to, and root of the light block merkle tree.
    pub fn run(self) -> ((CryptoHash, u64), CryptoHash) {
        print_file_path_and_function_name!();

        let file = File::open(Path::new(self.json_file_path.as_str()))
            .with_context(|| "Could not open proof file.")
            .unwrap();
        let reader = BufReader::new(file);
        let light_client_rpc_response: Value =
            serde_json::from_reader(reader).with_context(|| "Failed to deserialize JSON.").unwrap();
        Self::verify_json(light_client_rpc_response).unwrap()
    }

    pub fn verify_json(
        light_client_rpc_response: Value,
    ) -> Result<((CryptoHash, u64), CryptoHash), VerifyProofError> {
        print_file_path_and_function_name!();

        let light_client_proof: RpcLightClientExecutionProofResponse =
            serde_json::from_value(light_client_rpc_response["result"].clone()).unwrap();

        println!(
            "Verifying light client proof for txn id: {:?}",
            light_client_proof.outcome_proof.id
        );
        let outcome_hashes = light_client_proof.outcome_proof.to_hashes();
        println!("Hashes of the outcome are: {:?}", outcome_hashes);

        let outcome_hash = CryptoHash::hash_borsh(&outcome_hashes);
        println!("Hash of the outcome is: {:?}", outcome_hash);

        let outcome_shard_root =
            compute_root_from_path(&light_client_proof.outcome_proof.proof, outcome_hash);
        println!("Shard outcome root is: {:?}", outcome_shard_root);
        let block_outcome_root = compute_root_from_path(
            &light_client_proof.outcome_root_proof,
            CryptoHash::hash_borsh(outcome_shard_root),
        );
        println!("Block outcome root is: {:?}", block_outcome_root);

        if light_client_proof.block_header_lite.inner_lite.outcome_root != block_outcome_root {
            println!(
                "{}",
                ansi_term::Colour::Red.bold().paint(format!(
                    "ERROR: computed outcome root: {:?} doesn't match the block one {:?}.",
                    block_outcome_root,
                    light_client_proof.block_header_lite.inner_lite.outcome_root
                ))
            );
            return Err(VerifyProofError::InvalidOutcomeRootProof);
        }
        let block_hash = light_client_proof.outcome_proof.block_hash;

        if light_client_proof.block_header_lite.hash()
            != light_client_proof.outcome_proof.block_hash
        {
            println!("{}",
                     ansi_term::Colour::Red.bold().paint(format!(
                         "ERROR: block hash from header lite {:?} doesn't match the one from outcome proof {:?}",
                         light_client_proof.block_header_lite.hash(),
                         light_client_proof.outcome_proof.block_hash
                     )));
            return Err(VerifyProofError::InvalidBlockHashProof);
        } else {
            println!(
                "{}",
                ansi_term::Colour::Green
                    .bold()
                    .paint(format!("Block hash matches {:?}", block_hash))
            );
        }

        // And now check that block exists in the light client.

        let light_block_merkle_root =
            compute_root_from_path(&light_client_proof.block_proof, block_hash);

        println!(
            "Please verify that your light block has the following block merkle root: {:?}",
            light_block_merkle_root
        );
        println!(
            "OR verify that block with this hash {:?} is in the chain at this heigth {:?}",
            block_hash, light_client_proof.block_header_lite.inner_lite.height
        );
        Ok((
            (block_hash, light_client_proof.block_header_lite.inner_lite.height),
            light_block_merkle_root,
        ))
    }
}

fn make_env_filter(verbose: Option<&str>) -> Result<EnvFilter, BuildEnvFilterError> {
    print_file_path_and_function_name!();

    let env_filter = EnvFilterBuilder::from_env().verbose(verbose).finish()?;
    // Sandbox node can log to sandbox logging target via sandbox_debug_log host function.
    // This is hidden by default so we enable it for sandbox node.
    let env_filter = if cfg!(feature = "sandbox") {
        env_filter.add_directive("sandbox=debug".parse().unwrap())
    } else {
        env_filter
    };
    Ok(env_filter)
}

#[derive(clap::Parser)]
pub(super) struct ValidateConfigCommand {}

impl ValidateConfigCommand {
    pub(super) fn run(&self, home_dir: &Path) -> anyhow::Result<()> {
        print_file_path_and_function_name!();

        nearcore::config::load_config(home_dir, GenesisValidationMode::Full)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{CryptoHash, NeardCmd, NeardSubCommand, VerifyProofError, VerifyProofSubCommand};
    use clap::Parser;
    use std::str::FromStr;

    #[test]
    fn optional_values() {
        let cmd = NeardCmd::parse_from(&["test", "init", "--chain-id=testid", "--fast"]);
        if let NeardSubCommand::Init(scmd) = cmd.subcmd {
            assert_eq!(scmd.chain_id, Some("testid".to_string()));
            assert!(scmd.fast);
        } else {
            panic!("incorrect subcommand");
        }
    }

    #[test]
    fn equal_no_value_syntax() {
        assert!(NeardCmd::try_parse_from(&[
            "test",
            "init",
            // * This line currently fails to be parsed (= without a value)
            "--chain-id=",
            "--test-seed=alice.near",
            "--account-id=test.near",
            "--fast"
        ])
            .is_err());
    }

    #[test]
    fn verify_proof_test() {
        assert_eq!(
            VerifyProofSubCommand::verify_json(
                serde_json::from_slice(include_bytes!("../res/proof_example.json")).unwrap()
            )
                .unwrap(),
            (
                (
                    CryptoHash::from_str("HqZHDTHSqH6Az22SZgFUjodGFDtfC2qSt4v9uYFpLuFC").unwrap(),
                    38 as u64
                ),
                CryptoHash::from_str("BWwZdhAhjAgKxZ5ycqn1CvXads5DjPMfj4kRdc1rWit8").unwrap()
            )
        );

        // Proof with a wrong outcome (as user specified wrong shard).
        assert_eq!(
            VerifyProofSubCommand::verify_json(
                serde_json::from_slice(include_bytes!("../res/invalid_proof.json")).unwrap()
            )
                .unwrap_err(),
            VerifyProofError::InvalidOutcomeRootProof
        );
    }
}
