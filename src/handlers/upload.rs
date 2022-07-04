use std::{io::Cursor, process::Stdio, sync::atomic::Ordering};

use askama::Template;
use axum::{extract::Multipart, http::StatusCode, response::Html, Extension};
use infer::MatcherType;
use sqlx::SqlitePool;
use tokio::{
    fs::File,
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    process::Command,
};
use uuid::Uuid;

use crate::{auth::Auth, database, error::Error};

#[tracing::instrument(skip(_auth), err)]
pub(crate) async fn get(_auth: Auth) -> Result<Html<String>, Error> {
    #[derive(Template)]
    #[template(path = "upload.html")]
    struct Page {}

    Ok(Html(Page {}.render()?))
}

#[tracing::instrument(skip(_auth, pool, multipart), err)]
pub(crate) async fn post(
    _auth: Auth,
    Extension(pool): Extension<SqlitePool>,
    mut multipart: Multipart,
) -> Result<StatusCode, Error> {
    while let Some(mut field) = multipart.next_field().await? {
        let name = field.name().unwrap().to_string();
        let ext = name.split('.').last().unwrap();

        let id = Uuid::new_v4();

        let path = std::env::current_dir()?
            .join("assets")
            .join("video")
            .join(format!("{}.{}", id, ext));

        {
            let mut file = File::create(&path).await?;

            while let Some(mut chunk) = field.chunk().await? {
                file.write_all_buf(&mut chunk).await?;
            }

            file.flush().await?;
        }

        let typ = get_type(&path).await?;
        let ext = typ.extension();

        let old_path = path;
        let path = std::env::current_dir()?
            .join("assets")
            .join("video")
            .join(format!("{}.{}", id, ext));

        tokio::fs::rename(&old_path, &path).await?;

        generate_thumbnail(&id, &path).await?;

        sqlx::query!("INSERT INTO videos(id, ext) VALUES (?, ?)", id, ext)
            .execute(&pool)
            .await?;
    }

    database::DB_GET_ALL_VIDEOS_CACHE_INVALIDATE.store(true, Ordering::Release);

    Ok(StatusCode::CREATED)
}

#[tracing::instrument(skip(path), fields(path = %path.as_ref().display()), err)]
async fn get_type<P: AsRef<std::path::Path>>(path: P) -> Result<infer::Type, Error> {
    let file = File::open(&path).await?;

    let limit = file
        .metadata()
        .await
        .map(|m| std::cmp::min(m.len(), 8192) as usize + 1)
        .unwrap_or(0);

    let mut bytes = Vec::with_capacity(limit);
    file.take(limit as u64).read_to_end(&mut bytes).await?;

    let typ = infer::get(&bytes).ok_or(Error::Infer(
        "unable to figure out mime type, buf may be empty",
    ))?;

    if typ.matcher_type() != MatcherType::Video {
        return Err(Error::InvalidFileType);
    }

    Ok(typ)
}

#[tracing::instrument(skip(path), fields(path = %path.as_ref().display()), err)]
async fn generate_thumbnail<P: AsRef<std::path::Path>>(id: &Uuid, path: P) -> Result<(), Error> {
    let bytes = get_webp_frame(path, 16).await?;

    let img =
        image::io::Reader::with_format(Cursor::new(bytes), image::ImageFormat::WebP).decode()?;
    let img = img.thumbnail(1920 / 5, 1080 / 5);

    let path = std::env::current_dir()?
        .join("assets")
        .join("images")
        .join(format!("{}.webp", id));

    img.save_with_format(path, image::ImageFormat::WebP)?;

    Ok(())
}

#[tracing::instrument(skip(path, index), err)]
async fn get_webp_frame<P: AsRef<std::path::Path>>(path: P, index: usize) -> Result<Vec<u8>, Error> {
    let child = Command::new("ffmpeg")
        .args([
            "-loglevel",
            "panic",
            "-i",
            path.as_ref().as_os_str().to_str().unwrap(),
            "-vf",
            format!("select=eq(n\\,{})", index).as_str(),
            "-vframes",
            "1",
            "-c:v",
            "webp",
            "-movflags",
            "empty_moov",
            "-f",
            "image2pipe",
            "pipe:1",
        ])
        .stdout(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output().await?;
    if output.status.success() && !output.stdout.is_empty() {
        Ok(output.stdout)
    } else {
        Err(Error::Ffmpeg(
            String::from_utf8_lossy(&output.stderr[..]).to_string(),
        ))
    }
}
