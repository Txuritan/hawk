mod admin;
mod assets;
mod index;
mod login;
mod upload;
mod video;

use axum::{routing::get, Router};

pub fn routes() -> Router {
    Router::new()
        .route("/", get(index::get).post(index::get))
        .route("/admin", get(admin::get))
        .route("/assets/:name", get(assets::style_script_get))
        .route("/assets/images/:id", get(assets::images_get))
        .route("/login", get(login::get).post(login::post))
        .route("/upload", get(upload::get).post(upload::post))
        .route("/video/:id", get(video::get))
}
