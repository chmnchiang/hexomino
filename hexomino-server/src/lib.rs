#![feature(array_methods)]
#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(let_else)]
#![feature(try_blocks)]

use axum::{
    routing::{get_service, IntoMakeService},
    Extension, Router,
};
use hyper::StatusCode;
use kernel::Kernel;
use sqlx::PgPool;
use tower_http::{services::ServeDir, trace::TraceLayer};

mod auth;
mod http;
mod kernel;
mod result;
mod utils;
mod ws;

#[cfg(feature = "competition-mode")]
type DbPool = PgPool;

#[cfg(feature = "competition-mode")]
pub async fn make_app() -> IntoMakeService<Router> {
    let database_url = std::env::var("DATABASE_URL").expect("cannot find DATABASE_URL in env");
    let pool = DbPool::connect(&database_url)
        .await
        .expect("fail to create DB pool");
    Kernel::init(pool.clone());
    Router::new()
        .nest("/api", http::routes())
        .nest("/ws", ws::routes())
        .nest(
            "/hexomino",
            get_service(ServeDir::new("dist")).handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            }),
        )
        .layer(Extension(pool))
        .layer(TraceLayer::new_for_http())
        .into_make_service()
}

#[cfg(not(feature = "competition-mode"))]
pub async fn make_app() -> IntoMakeService<Router> {
    let dist_path = std::env::var("DIST_PATH");
    Kernel::init();
    let mut router = Router::new()
        .nest("/api", http::routes())
        .nest("/ws", ws::routes());
    if let Ok(path) = dist_path {
        router = router.nest(
            "/hexomino",
            get_service(ServeDir::new(path)).handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            }),
        );
    }

    router.layer(TraceLayer::new_for_http()).into_make_service()
}
