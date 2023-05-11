use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use wasmer::{CompilerConfig, Engine as WasmerEngine, Features, Store};

#[derive(Clone, Debug, PartialEq)]
pub enum Compiler {
    Singlepass,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Engine {
    Dylib,
    Universal,
}

#[derive(Clone)]
pub struct Config {
    pub compiler: Compiler,
    pub engine: Engine,
    pub features: Option<Features>,
    pub canonicalize_nans: bool,
}

impl Config {
    pub fn new(engine: Engine, compiler: Compiler) -> Self {
print_file_path_and_function_name!();

        Self {
            compiler,
            engine,
            features: None,
            canonicalize_nans: false,
        }
    }

    pub fn set_features(&mut self, features: Features) {
print_file_path_and_function_name!();

        self.features = Some(features);
    }

    pub fn set_nan_canonicalization(&mut self, canonicalize_nans: bool) {
print_file_path_and_function_name!();

        self.canonicalize_nans = canonicalize_nans;
    }

    pub fn store(&self) -> Store {
print_file_path_and_function_name!();

        let compiler_config = self.compiler_config(self.canonicalize_nans);
        let engine = self.engine(compiler_config);
        Store::new(&*engine)
    }

    pub fn headless_store(&self) -> Store {
print_file_path_and_function_name!();

        let engine = self.engine_headless();
        Store::new(&*engine)
    }

    pub fn engine(&self, compiler_config: Box<dyn CompilerConfig>) -> Box<dyn WasmerEngine> {
print_file_path_and_function_name!();

        #[cfg(not(feature = "engine"))]
        compile_error!("Plese enable at least one engine via the features");
        match &self.engine {
            #[cfg(feature = "universal")]
            Engine::Universal => {
                let mut engine = wasmer_engine_universal::Universal::new(compiler_config);
                if let Some(ref features) = self.features {
                    engine = engine.features(features.clone())
                }
                Box::new(engine.engine())
            }
            #[allow(unreachable_patterns)]
            engine => panic!(
                "The {:?} Engine is not enabled. Please enable it using the features",
                engine
            ),
        }
    }

    pub fn engine_headless(&self) -> Box<dyn WasmerEngine> {
print_file_path_and_function_name!();

        match &self.engine {
            #[cfg(feature = "universal")]
            Engine::Universal => Box::new(wasmer_engine_universal::Universal::headless().engine()),
            #[allow(unreachable_patterns)]
            engine => panic!(
                "The {:?} Engine is not enabled. Please enable it using the features",
                engine
            ),
        }
    }

    pub fn compiler_config(&self, canonicalize_nans: bool) -> Box<dyn CompilerConfig> {
print_file_path_and_function_name!();

        match &self.compiler {
            #[cfg(feature = "singlepass")]
            Compiler::Singlepass => {
                let mut compiler = wasmer_compiler_singlepass::Singlepass::new();
                compiler.canonicalize_nans(canonicalize_nans);
                compiler.enable_verifier();
                Box::new(compiler)
            }
            #[allow(unreachable_patterns)]
            compiler => {
                panic!(
                    "The {:?} Compiler is not enabled. Enable it via the features",
                    compiler
                )
            }
        }
    }
}
