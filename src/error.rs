#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid hostname: {0}")]
    InvalidHostname(String),
}