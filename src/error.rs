use axum::response::IntoResponse;
use http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("address parse: {0}")]
    AddrParse(#[from] std::net::AddrParseError),
    #[error("askama: {0}")]
    Askama(#[from] askama::Error),
    #[error("bcrypt: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
    #[error("ffmpeg: {0}")]
    Ffmpeg(String),
    #[error("image: {0}")]
    Image(#[from] image::error::ImageError),
    #[error("infer: {0}")]
    Infer(&'static str),
    #[error("io: {0}")]
    Io(#[from] tokio::io::Error),
    #[error("invalid file type uploaded")]
    InvalidFileType,
    #[error("multipart: {0}")]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error("sqlx: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("sqlx migration: {0}")]
    SqlMigrate(#[from] sqlx::migrate::MigrateError),
    #[error("tokio join: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("webp encoding: {0}")]
    Webp(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{}", self);

        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
