use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


/// Proto conversion utilities.
use protobuf::MessageField as MF;

#[derive(thiserror::Error, Debug)]
#[error("[{idx}]: {source}")]
pub struct ParseVecError<E> {
    idx: usize,
    #[source]
    source: E,
}

pub fn try_from_slice<'a, X, Y: TryFrom<&'a X>>(
    xs: &'a [X],
) -> Result<Vec<Y>, ParseVecError<Y::Error>> {
print_file_path_and_function_name!();

    let mut ys = vec![];
    for (idx, x) in xs.iter().enumerate() {
        ys.push(x.try_into().map_err(|source| ParseVecError { idx, source })?);
    }
    Ok(ys)
}

#[derive(thiserror::Error, Debug)]
pub enum ParseRequiredError<E> {
    #[error("missing, while required")]
    Missing,
    #[error(transparent)]
    Other(E),
}

pub fn try_from_optional<'a, X, Y: TryFrom<&'a X>>(x: &'a MF<X>) -> Result<Option<Y>, Y::Error> {
print_file_path_and_function_name!();

    x.as_ref().map(|x| x.try_into()).transpose()
}

pub fn try_from_required<'a, X, Y: TryFrom<&'a X>>(
    x: &'a MF<X>,
) -> Result<Y, ParseRequiredError<Y::Error>> {
print_file_path_and_function_name!();

    x.as_ref().ok_or(ParseRequiredError::Missing)?.try_into().map_err(ParseRequiredError::Other)
}

pub fn map_from_required<'a, X, Y, E>(
    x: &'a MF<X>,
    f: impl FnOnce(&'a X) -> Result<Y, E>,
) -> Result<Y, ParseRequiredError<E>> {
print_file_path_and_function_name!();

    f(x.as_ref().ok_or(ParseRequiredError::Missing)?).map_err(ParseRequiredError::Other)
}
