use axum::{extract::Path, response::IntoResponse};
use http::{header, StatusCode};
use tokio::{fs::File, io::AsyncReadExt as _};

use crate::{
    error::Error,
    response::{Css, Either, Js, Left, Right},
    AXIOS_JS, STYLE_CSS,
};

#[tracing::instrument(skip(name))]
pub(crate) async fn style_script_get(
    Path(name): Path<String>,
) -> Either<Either<Css, Js>, StatusCode> {
    match name.as_str() {
        "axios.min.js" => Left(Right(Js(AXIOS_JS))),
        "style.css" => Left(Left(Css(STYLE_CSS))),
        _ => Right(StatusCode::NOT_FOUND),
    }
}

#[tracing::instrument(skip(id))]
pub(crate) async fn images_get(
    Path(id): Path<String>,
) -> Result<Either<impl IntoResponse, StatusCode>, Error> {
    let image_path = std::env::current_dir()?
        .join("assets")
        .join("images")
        .join(id);

    if !image_path.exists() {
        return Ok(Right(StatusCode::NOT_FOUND));
    }

    let mut image_file = File::open(&image_path).await?;

    let mut bytes = Vec::with_capacity(1024 * 1024);

    image_file.read_to_end(&mut bytes).await?;

    Ok(Left(
        (
            StatusCode::OK,
            [
                (header::CACHE_CONTROL, "public, max-age=604800"),
                (header::CONTENT_TYPE, "image/webp"),
            ],
            bytes,
        )
            .into_response(),
    ))
}
