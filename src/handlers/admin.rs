use askama::Template;
use axum::{response::Html, Extension, Form};
use http::StatusCode;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{auth::Auth, database, error::Error};

pub(crate) async fn get(
    _auth: Auth,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Html<String>, Error> {
    #[derive(Template)]
    #[template(path = "admin.html")]
    struct Page {
        videos: Vec<Uuid>,
    }

    let videos = database::db_get_all_videos(pool).await?;

    Ok(Html(Page { videos }.render()?))
}

pub(crate) async fn clear_sessions(
    _auth: Auth,
    Extension(pool): Extension<SqlitePool>,
) -> Result<StatusCode, Error> {
    sqlx::query!("DELETE FROM sessions;").execute(&pool).await?;

    Ok(StatusCode::OK)
}

#[derive(serde::Deserialize)]
pub(crate) struct RemoveVideo {
    id: String,
}

pub(crate) async fn remove_video(
    _auth: Auth,
    Extension(pool): Extension<SqlitePool>,
    Form(form): Form<RemoveVideo>,
) -> Result<StatusCode, Error> {
    let mut trans = pool.begin().await?;

    sqlx::query!("DELETE FROM videos WHERE id = ?;", form.id)
        .execute(&mut trans)
        .await?;

    let path = std::env::current_dir()?
        .join("assets")
        .join("video")
        .join(format!("{}.webp", form.id));
    tokio::fs::remove_file(&path).await?;

    trans.commit().await?;

    Ok(StatusCode::OK)
}
