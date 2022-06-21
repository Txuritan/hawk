use std::sync::atomic::Ordering;

use askama::Template;
use axum::{
    response::{Html, Redirect},
    Extension, Form,
};
use cookie::{Cookie, SameSite};
use sqlx::SqlitePool;
use time::{ext::NumericalDuration as _, Duration, OffsetDateTime};
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::{database, error::Error, models::Login, SESSION};

#[tracing::instrument(err)]
pub(crate) async fn get() -> Result<Html<String>, Error> {
    #[derive(Template)]
    #[template(path = "login.html")]
    struct Page {}

    Ok(Html(Page {}.render()?))
}

#[tracing::instrument(skip(pool, login), err)]
pub(crate) async fn post(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(login): Form<Login>,
) -> Result<Redirect, Error> {
    if !login.email.is_empty() {
        return Ok(Redirect::permanent("/"));
    }

    let user = sqlx::query!(
        r#"SELECT id as "id: Uuid", hash FROM users WHERE username = ? "#,
        login.username
    )
    .fetch_one(&pool)
    .await?;
    let valid = bcrypt::verify(&login.password, &user.hash)?;

    if !valid {
        return Ok(Redirect::permanent("/"));
    }

    let token = nanoid::nanoid!(64);

    sqlx::query!(
        "INSERT INTO sessions(id, token) VALUES (?, ?)",
        user.id,
        token
    )
    .execute(&pool)
    .await?;

    cookies.add(
        Cookie::build(SESSION, token)
            .path("/")
            .secure(true)
            .http_only(true)
            .same_site(SameSite::Strict)
            .max_age(Duration::days(7))
            .expires(OffsetDateTime::now_utc() + 7.days())
            .finish(),
    );

    database::DB_GET_ALL_VIDEOS_CACHE_INVALIDATE.store(true, Ordering::Release);

    Ok(Redirect::permanent("/"))
}
