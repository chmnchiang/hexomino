use hexomino_server::all_routes;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "hexomino-server=debug,tower_http=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(all_routes().into_make_service())
        .await
        .unwrap();
}
