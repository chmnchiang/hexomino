use hexomino_server::make_app;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "hexomino_server=trace,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(make_app().await)
        .await
        .unwrap();
}
