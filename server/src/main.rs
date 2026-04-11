use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

mod game_session;
mod lobby;
mod protocol;
mod room;
mod ws_handler;

use lobby::RoomManager;
use ws_handler::{ws_upgrade, AppState};

#[tokio::main]
async fn main() {
    let state: AppState = Arc::new(Mutex::new(RoomManager::new()));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/ws", get(ws_upgrade))
        .layer(cors)
        .fallback_service(ServeDir::new("./public"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    println!("Penguin Soccer server listening on 0.0.0.0:3000");

    axum::serve(listener, app).await.expect("Server error");
}
