use axum::Router;
use tower_http::trace::TraceLayer;

mod api;

pub fn all_routes() -> Router {
    Router::new()
        .nest("/api", api::routes())
        .layer(TraceLayer::new_for_http())
}
