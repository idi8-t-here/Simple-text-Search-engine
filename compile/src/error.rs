use thiserror::Error;

#[derive(Debug, Error)]
pub enum Errors {
    #[error("unable to find the folder")]
    Variant1,
    #[error("unable to serialize the tree")]
    Variant1,
}
