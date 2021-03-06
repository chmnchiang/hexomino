use axum::{extract::WebSocketUpgrade, response::IntoResponse, routing::get, Router};


use crate::kernel::Kernel;

pub fn routes() -> Router {
    Router::new().route("/", get(start_ws))
}

async fn start_ws(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |ws| async {
        Kernel::get().new_connection(ws).await;
    })
}
