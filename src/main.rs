use axum::{routing::get, Router};
use std::{net::SocketAddr, path::PathBuf};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod game;
mod types;
mod ui;

use types::AppState;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    let state = std::sync::Arc::new(tokio::sync::Mutex::new(AppState::new()));
    state.lock().await.add_room("test");

    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .nest("", ui::pages_router())
        .nest("/c", ui::components_router())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
