mod auth;
mod db;
mod handlers;
mod models;

use handlers::auth::AppState;
use handlers::{auth as auth_handlers, board, invoices};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chaser=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:chaser.db".to_string());
    let base_url =
        std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let pool = db::create_pool(&database_url).await?;
    let state = AppState { pool, base_url };

    let app = axum::Router::new()
        .route("/", axum::routing::get(auth_handlers::login_page))
        .route(
            "/auth/magic-link",
            axum::routing::post(auth_handlers::request_magic_link),
        )
        .route("/auth/verify", axum::routing::get(auth_handlers::verify))
        .route("/logout", axum::routing::get(auth_handlers::logout))
        .route("/board", axum::routing::get(board::board))
        .route("/invoices", axum::routing::post(invoices::create_invoice))
        .route(
            "/invoices/:id/chase",
            axum::routing::post(invoices::send_chase),
        )
        .route(
            "/invoices/:id/paid",
            axum::routing::post(invoices::mark_paid),
        )
        .route(
            "/invoices/:id/snooze",
            axum::routing::post(invoices::snooze),
        )
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Chaser running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
