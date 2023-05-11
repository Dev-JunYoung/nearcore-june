use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use super::ValidatedOperation;

pub(crate) struct SignedDelegateActionOperation {
    pub(crate) receiver_id: crate::models::AccountIdentifier,
    pub(crate) signature: near_crypto::Signature,
}

impl ValidatedOperation for SignedDelegateActionOperation {
    const OPERATION_TYPE: crate::models::OperationType =
        crate::models::OperationType::SignedDelegateAction;

    fn into_operation(
        self,
        operation_identifier: crate::models::OperationIdentifier,
    ) -> crate::models::Operation {
print_file_path_and_function_name!();

        crate::models::Operation {
            operation_identifier,

            account: self.receiver_id,
            amount: None,
            metadata: Some(crate::models::OperationMetadata {
                signature: Some(self.signature.to_string()),
                ..Default::default()
            }),

            related_operations: None,
            type_: Self::OPERATION_TYPE,
            status: None,
        }
    }
}
fn required_fields_error() -> crate::errors::ErrorKind {
print_file_path_and_function_name!();

    crate::errors::ErrorKind::InvalidInput(
        "DELEGATE_ACTION operation requires `signature` being passed in the metadata".into(),
    )
}
impl TryFrom<crate::models::Operation> for SignedDelegateActionOperation {
    type Error = crate::errors::ErrorKind;

    fn try_from(operation: crate::models::Operation) -> Result<Self, Self::Error> {
print_file_path_and_function_name!();

        Self::validate_operation_type(operation.type_)?;
        let metadata = operation.metadata.ok_or_else(required_fields_error)?;
        let signature =
            metadata.signature.ok_or_else(required_fields_error)?.parse().map_err(|_| {
                crate::errors::ErrorKind::InvalidInput("Invalid key format".to_string())
            })?;

        Ok(Self { receiver_id: operation.account, signature })
    }
}
