
#[cfg(feature = "test_features")]
mod adv {
    use blockbuster::Utc;
    use blockbuster::DepthGuard;
    use blockbuster::DEPTH_COUNTER;
    use blockbuster::TOTAL_COUNTER;
    use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;

    use std::sync::atomic::Ordering;

    #[derive(Default)]
    struct Inner {
        disable_header_sync: std::sync::atomic::AtomicBool,
        disable_doomslug: std::sync::atomic::AtomicBool,
        is_archival: bool,
    }

    #[derive(Default, Clone)]
    pub struct Controls(std::sync::Arc<Inner>);

    impl Controls {
        pub fn new(is_archival: bool) -> Self {
            print_file_path_and_function_name!();
            Self(std::sync::Arc::new(Inner { is_archival, ..Inner::default() }))
        }

        pub fn disable_header_sync(&self) -> bool {
            print_file_path_and_function_name!();
            self.0.disable_header_sync.load(Ordering::SeqCst)
        }

        pub fn set_disable_header_sync(&mut self, value: bool) {
            print_file_path_and_function_name!();
            self.0.disable_header_sync.store(value, Ordering::SeqCst);
        }

        pub fn disable_doomslug(&self) -> bool {
            print_file_path_and_function_name!();
            self.0.disable_doomslug.load(Ordering::SeqCst)
        }

        pub fn set_disable_doomslug(&self, value: bool) {
            print_file_path_and_function_name!();
            self.0.disable_doomslug.store(value, Ordering::SeqCst);
        }

        pub fn is_archival(&self) -> bool {
            print_file_path_and_function_name!();
            self.0.is_archival
        }
    }
}

#[cfg(not(feature = "test_features"))]
mod adv {
    #[derive(Default, Clone)]
    pub struct Controls;

    impl Controls {
        pub const fn new(_is_archival: bool) -> Self {
            Self
        }

        pub const fn disable_header_sync(&self) -> bool {
            false
        }

        pub const fn disable_doomslug(&self) -> bool {
            false
        }
    }
}

pub use adv::Controls;
