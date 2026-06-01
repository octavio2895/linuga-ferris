#[derive(Debug, thiserror::Error)]
pub enum LinguaError {
    #[error("API request failed: {0}")]
    Api(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("Environment variable error: {0}")]
    Env(#[from] std::env::VarError),

    #[error("Empty response from API")]
    EmptyResponse,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
