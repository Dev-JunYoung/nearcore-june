use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use super::ValidatedOperation;

pub(crate) struct RefundDeleteAccountOperation {
    pub(crate) beneficiary_account: crate::models::AccountIdentifier,
}

impl super::ValidatedOperation for RefundDeleteAccountOperation {
    const OPERATION_TYPE: crate::models::OperationType =
        crate::models::OperationType::RefundDeleteAccount;

    fn into_operation(
        self,
        operation_identifier: crate::models::OperationIdentifier,
    ) -> crate::models::Operation {
print_file_path_and_function_name!();

        crate::models::Operation {
            operation_identifier,

            account: self.beneficiary_account,
            amount: None,
            metadata: None,

            related_operations: None,
            type_: Self::OPERATION_TYPE,
            status: None,
        }
    }
}

impl TryFrom<crate::models::Operation> for RefundDeleteAccountOperation {
    type Error = crate::errors::ErrorKind;

    fn try_from(operation: crate::models::Operation) -> Result<Self, Self::Error> {
print_file_path_and_function_name!();

        Self::validate_operation_type(operation.type_)?;
        Ok(Self { beneficiary_account: operation.account })
    }
}
