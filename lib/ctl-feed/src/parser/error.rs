use thiserror::Error;

#[derive(Debug, Error)]
pub enum DummyParserError {
    #[error("dummy parser error")]
    General,
}