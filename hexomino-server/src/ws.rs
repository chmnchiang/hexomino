use std::sync::Arc;

use axum::{extract::WebSocketUpgrade, response::IntoResponse, routing::get, Extension, Router};
use tracing::debug;

use crate::kernel::Kernel;

pub fn routes() -> Router {
    Router::new().route("/", get(start_ws))
}

async fn start_ws(
    ws: WebSocketUpgrade,
    Extension(kernel): Extension<Arc<Kernel>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |ws| async {
        debug!("websocket upgrade");
        kernel.new_connection(ws).await;
    })
}
