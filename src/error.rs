use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP Error: {source:?}")]
    Http {
        #[from]
        source: reqwest::Error,
    },
    #[error("JSON Error in {}: {}", .source.path(), .source.inner())]
    Json {
        #[from]
        source: serde_path_to_error::Error<serde_json::Error>,
    },
    #[error("Url parse error: {source:?}")]
    Url {
        #[from]
        source: url::ParseError,
    },
}
