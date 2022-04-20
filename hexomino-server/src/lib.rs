use axum::{routing::IntoMakeService, Extension, Router};
use kernel::Kernel;
use sqlx::SqlitePool;
use tower_http::trace::TraceLayer;

mod api;
mod kernel;
mod ws;

type DbPool = SqlitePool;

pub async fn make_app() -> IntoMakeService<Router> {
    let database_url = std::env::var("DATABASE_URL").expect("cannot find DATABASE_URL in env");
    let pool = DbPool::connect(&database_url)
        .await
        .expect("fail to create DB pool");

    Router::new()
        .nest("/api", api::routes())
        .nest("/ws", ws::routes())
        .layer(Extension(pool.clone()))
        .layer(Extension(Kernel::new(pool)))
        .layer(TraceLayer::new_for_http())
        .into_make_service()
}
