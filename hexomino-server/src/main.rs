use hexomino_server::make_app;
use tracing_subscriber::{
    filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

#[cfg(feature = "competition-mode")]
#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    let file_appender = tracing_appender::rolling::hourly("./logs", "server.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);
    let file_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .with_env_var("FILE_LOG_LEVEL")
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "hexomino_server=debug,tower_http=debug".into()),
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file_writer)
                .with_filter(file_filter),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .init();
    axum::Server::bind(
        &std::env::var("SERVER_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:3000".into())
            .parse()
            .expect("failed to parse server address"),
    )
    .serve(make_app().await)
    .await
    .expect("failed to start server");
}

#[cfg(not(feature = "competition-mode"))]
#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "hexomino_server=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .init();
    axum::Server::bind(
        &std::env::var("SERVER_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:3000".into())
            .parse()
            .expect("failed to parse server address"),
    )
    .serve(make_app().await)
    .await
    .expect("failed to start server");
}
