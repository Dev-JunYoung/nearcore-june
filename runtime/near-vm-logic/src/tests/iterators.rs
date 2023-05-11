use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use crate::tests::vm_logic_builder::VMLogicBuilder;
use near_vm_errors::{HostError, VMLogicError};

#[test]
fn test_iterator_deprecated() {
    let mut logic_builder = VMLogicBuilder::default();
    let mut logic = logic_builder.build();
    assert_eq!(
        Err(VMLogicError::HostError(HostError::Deprecated {
            method_name: "storage_iter_prefix".to_string()
        })),
        logic.storage_iter_prefix(1, b"a".as_ptr() as _)
    );
    assert_eq!(
        Err(VMLogicError::HostError(HostError::Deprecated {
            method_name: "storage_iter_range".to_string()
        })),
        logic.storage_iter_range(1, b"a".as_ptr() as _, 1, b"b".as_ptr() as _)
    );
    assert_eq!(
        Err(VMLogicError::HostError(HostError::Deprecated {
            method_name: "storage_iter_next".to_string()
        })),
        logic.storage_iter_next(0, 0, 1)
    );
}
