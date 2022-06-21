use axum::response::IntoResponse;
use http::{header, StatusCode};

pub use Either::*;

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> IntoResponse for Either<L, R>
where
    L: IntoResponse,
    R: IntoResponse,
{
    fn into_response(self) -> axum::response::Response {
        match self {
            Either::Left(side) => side.into_response(),
            Either::Right(side) => side.into_response(),
        }
    }
}

pub struct Css(pub &'static str);

impl axum::response::IntoResponse for Css {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::OK,
            [
                (header::CACHE_CONTROL, "public, max-age=604800"),
                (header::CONTENT_TYPE, "text/css; charset=UTF-8"),
            ],
            self.0,
        )
            .into_response()
    }
}

pub struct Js(pub &'static str);

impl axum::response::IntoResponse for Js {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::OK,
            [
                (header::CACHE_CONTROL, "public, max-age=604800"),
                (header::CONTENT_TYPE, "text/javascript; charset=UTF-8"),
            ],
            self.0,
        )
            .into_response()
    }
}
