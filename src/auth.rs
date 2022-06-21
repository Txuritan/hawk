use axum::{
    extract::{FromRequest, RequestParts},
    Extension,
};
use http::StatusCode;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

use crate::{database, SESSION};

pub(crate) struct Auth;

#[async_trait::async_trait]
impl<B: Send> FromRequest<B> for Auth {
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<SqlitePool>::from_request(req)
            .await
            .expect("`SqlitePool` extension missing");

        let cookie = Option::<Cookies>::from_request(req)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let session = cookie.get(SESSION).ok_or(StatusCode::UNAUTHORIZED)?;
        let value = session.value();

        if !database::db_is_session_valid(pool, value)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?
        {
            return Err(StatusCode::UNAUTHORIZED);
        }

        Ok(Auth)
    }
}
