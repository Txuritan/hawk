use askama::Template;
use axum::{extract::Path, response::Html, Extension};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{auth::Auth, error::Error, models::Video};

#[tracing::instrument(skip(_auth, pool), err)]
pub(crate) async fn get(
    _auth: Auth,
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<Uuid>,
) -> Result<Html<String>, Error> {
    #[derive(Template)]
    #[template(path = "video.html")]
    struct Page {
        video: Video,
    }

    let video = sqlx::query_as!(
        Video,
        r#"SELECT id as "id: Uuid", ext FROM videos WHERE id = ?"#,
        id
    )
    .fetch_one(&pool)
    .await?;

    Ok(Html(Page { video }.render()?))
}
