mod handlers;

mod auth;
mod database;
mod error;
mod models;
mod response;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use axum::{Extension, Router};
use axum_extra::routing::SpaRouter;
use axum_server::{tls_rustls::RustlsConfig, Handle};
use clap::Parser as _;
use sqlx::SqlitePool;
use tokio::{fs::File, io::AsyncWriteExt as _};
use tower_cookies::CookieManagerLayer;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

use crate::error::Error;

const SESSION: &str = "hawk-session";
const MIGRATIONS: sqlx::migrate::Migrator = sqlx::migrate!();

static AXIOS_JS: &str = include_str!("../node_modules/axios/dist/axios.min.js");
static STYLE_CSS: &str = include_str!("../assets/style.css");

// convert images to webp: (for %i in (*.png) do ffmpeg -i %i %~ni.webp)

#[derive(clap::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The host the server should bind to
    #[clap(short, long, value_parser, default_value = "0.0.0.0")]
    host: String,

    /// The port the server should listen to
    #[clap(short, long, value_parser, default_value_t = 25575)]
    port: u16,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "hawk=debug,sqlx=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime")
        .block_on(entry(args))?;

    Ok(())
}

async fn entry(args: Args) -> Result<(), Error> {
    let db = std::env::current_dir()?.join("hawk.db");
    if !db.exists() {
        let mut db = File::create(&db).await?;
        db.flush().await?;
    }

    let pool = SqlitePool::connect("sqlite://hawk.db").await?;

    MIGRATIONS.run(&pool).await?;

    // {
    //     let id = Uuid::new_v4();
    //     let hash = bcrypt::hash("******", bcrypt::DEFAULT_COST)?;
    //     sqlx::query!("INSERT INTO users(id, username, hash) VALUES (?, ?, ?)", id, "******", hash).execute(&pool).await?;
    // }

    let handle = Handle::new();
    tokio::spawn(graceful_shutdown(handle.clone()));

    let config = RustlsConfig::from_pem_file("cert.pem", "key.pem")
        .await
        .unwrap();

    let app = Router::new()
        .merge(handlers::routes())
        .merge(SpaRouter::new("/assets/video", "./assets/video"))
        .layer(Extension(pool.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new())
        .layer(CompressionLayer::new());

    let addr = args.host.parse::<Ipv4Addr>()?;

    let addr = SocketAddr::from(SocketAddrV4::new(addr, args.port));
    tracing::info!("listening on {}", addr);
    axum_server::bind_rustls(addr, config)
        .handle(handle)
        .serve(app.into_make_service())
        .await
        .unwrap();

    pool.close().await;

    tracing::info!("graceful shutdown complete");

    Ok(())
}

async fn graceful_shutdown(handle: Handle) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");

    handle.graceful_shutdown(None);
}
