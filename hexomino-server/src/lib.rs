use axum::{routing::IntoMakeService, Extension, Router};
use kernel::Kernel;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

mod auth;
mod http;
mod kernel;
mod result;
mod utils;
mod ws;

type DbPool = PgPool;

pub async fn make_app() -> IntoMakeService<Router> {
    let database_url = std::env::var("DATABASE_URL").expect("cannot find DATABASE_URL in env");
    let pool = DbPool::connect(&database_url)
        .await
        .expect("fail to create DB pool");
    Kernel::init(pool.clone());

    Router::new()
        .nest("/api", http::routes())
        .nest("/ws", ws::routes())
        .layer(Extension(pool))
        .layer(TraceLayer::new_for_http())
        .into_make_service()
}
