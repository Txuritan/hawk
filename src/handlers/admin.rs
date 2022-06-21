use askama::Template;
use axum::{response::Html, Extension};
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
