use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub(crate) enum Error {
    #[error(transparent)]
    FindError(#[from] xplm::data::borrowed::FindError),
}
