use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RosettaRpcConfig {
    pub addr: String,
    pub cors_allowed_origins: Vec<String>,
    #[serde(default)]
    pub limits: RosettaRpcLimitsConfig,
}

impl Default for RosettaRpcConfig {
    fn default() -> Self {
print_file_path_and_function_name!();

        Self {
            addr: "0.0.0.0:3040".to_owned(),
            cors_allowed_origins: vec!["*".to_owned()],
            limits: RosettaRpcLimitsConfig::default(),
        }
    }
}

impl RosettaRpcConfig {
    pub fn new(addr: &str) -> Self {
print_file_path_and_function_name!();

        Self { addr: addr.to_owned(), ..Default::default() }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RosettaRpcLimitsConfig {
    pub input_payload_max_size: usize,
}

impl Default for RosettaRpcLimitsConfig {
    fn default() -> Self {
print_file_path_and_function_name!();

        Self { input_payload_max_size: 10 * 1024 * 1024 }
    }
}
