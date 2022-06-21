use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
};

use once_cell::sync::Lazy;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::Error;

pub(crate) static DB_GET_ALL_VIDEOS_CACHE_INVALIDATE: AtomicBool = AtomicBool::new(false);

pub(crate) async fn db_get_all_videos(pool: SqlitePool) -> Result<Vec<Uuid>, Error> {
    static DB_GET_ALL_VIDEOS_CACHE: Lazy<RwLock<Vec<Uuid>>> = Lazy::new(|| RwLock::new(Vec::new()));

    if DB_GET_ALL_VIDEOS_CACHE_INVALIDATE.load(Ordering::Acquire) {
        DB_GET_ALL_VIDEOS_CACHE.write().await.clear();
        DB_GET_ALL_VIDEOS_CACHE_INVALIDATE.store(false, Ordering::Release);
    }

    if DB_GET_ALL_VIDEOS_CACHE.read().await.is_empty() {
        let videos =
            sqlx::query_scalar!(r#"SELECT id as "id: Uuid" FROM videos ORDER BY created DESC"#)
                .fetch_all(&pool)
                .await?;

        *DB_GET_ALL_VIDEOS_CACHE.write().await = videos.clone();

        return Ok(videos);
    }

    let ids = DB_GET_ALL_VIDEOS_CACHE.read().await.clone();

    Ok(ids)
}

pub(crate) static DB_IS_SESSION_VALID_CACHE_INVALIDATE: AtomicBool = AtomicBool::new(false);

pub(crate) async fn db_is_session_valid(pool: SqlitePool, token: &str) -> Result<bool, Error> {
    static DB_IS_SESSION_VALID_CACHE: Lazy<RwLock<HashMap<String, bool>>> =
        Lazy::new(|| RwLock::new(HashMap::new()));

    if DB_IS_SESSION_VALID_CACHE_INVALIDATE.load(Ordering::Acquire) {
        DB_IS_SESSION_VALID_CACHE.write().await.clear();
        DB_IS_SESSION_VALID_CACHE_INVALIDATE.store(false, Ordering::Release);
    }

    if !DB_IS_SESSION_VALID_CACHE.read().await.contains_key(token) {
        let user_id = sqlx::query!(
            r#"SELECT id as "id: Uuid" FROM sessions WHERE token = ?"#,
            token
        )
        .fetch_optional(&pool)
        .await?;

        let is_valid = user_id.is_some();

        DB_IS_SESSION_VALID_CACHE
            .write()
            .await
            .insert(token.to_string(), is_valid);

        return Ok(is_valid);
    }

    Ok(DB_IS_SESSION_VALID_CACHE
        .read()
        .await
        .get(token)
        .copied()
        .unwrap_or(false))
}
