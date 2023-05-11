use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use super::ValidatedOperation;

pub(crate) struct AddKeyOperation {
    pub(crate) account: crate::models::AccountIdentifier,
    pub(crate) public_key: crate::models::PublicKey,
}

impl ValidatedOperation for AddKeyOperation {
    const OPERATION_TYPE: crate::models::OperationType = crate::models::OperationType::AddKey;

    fn into_operation(
        self,
        operation_identifier: crate::models::OperationIdentifier,
    ) -> crate::models::Operation {
print_file_path_and_function_name!();

        crate::models::Operation {
            operation_identifier,

            account: self.account,
            amount: None,
            metadata: Some(crate::models::OperationMetadata {
                public_key: Some(self.public_key),
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
        "ADD_KEY operation requires `public_key` being passed in the metadata".into(),
    )
}

impl TryFrom<crate::models::Operation> for AddKeyOperation {
    type Error = crate::errors::ErrorKind;

    fn try_from(operation: crate::models::Operation) -> Result<Self, Self::Error> {
print_file_path_and_function_name!();

        Self::validate_operation_type(operation.type_)?;
        let metadata = operation.metadata.ok_or_else(required_fields_error)?;
        let public_key = metadata.public_key.ok_or_else(required_fields_error)?;

        Ok(Self { account: operation.account, public_key })
    }
}
