use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use super::ValidatedOperation;

pub(crate) struct TransferOperation {
    pub(crate) account: crate::models::AccountIdentifier,
    pub(crate) amount: crate::models::Amount,
    pub(crate) predecessor_id: Option<crate::models::AccountIdentifier>,
}

impl ValidatedOperation for TransferOperation {
    const OPERATION_TYPE: crate::models::OperationType = crate::models::OperationType::Transfer;

    fn into_operation(
        self,
        operation_identifier: crate::models::OperationIdentifier,
    ) -> crate::models::Operation {
print_file_path_and_function_name!();

        crate::models::Operation {
            operation_identifier,

            account: self.account,
            amount: Some(self.amount),
            metadata: crate::models::OperationMetadata::from_predecessor(self.predecessor_id),

            related_operations: None,
            type_: Self::OPERATION_TYPE,
            status: None,
        }
    }
}

fn required_fields_error() -> crate::errors::ErrorKind {
print_file_path_and_function_name!();

    crate::errors::ErrorKind::InvalidInput(
        "TRANSFER operation requires `amount` being specified".into(),
    )
}

impl TryFrom<crate::models::Operation> for TransferOperation {
    type Error = crate::errors::ErrorKind;

    fn try_from(operation: crate::models::Operation) -> Result<Self, Self::Error> {
print_file_path_and_function_name!();

        Self::validate_operation_type(operation.type_)?;
        let amount = operation.amount.ok_or_else(required_fields_error)?;
        let predecessor_id = operation.metadata.and_then(|metadata| metadata.predecessor_id);
        Ok(Self { account: operation.account, amount, predecessor_id })
    }
}
