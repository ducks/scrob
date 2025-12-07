mod auth;
mod config;
mod db;
mod routes;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file if present
    let _ = dotenvy::dotenv();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "scrob=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config = Config::from_env()?;
    tracing::info!("Starting scrob server");
    tracing::info!("Database: {}", config.database_url);
    tracing::info!("Listening on: {}", config.bind_address());

    // Connect to database and run migrations
    let pool = db::create_pool(&config.database_url).await?;

    // Build router
    let app = Router::new()
        // Auth
        .route("/signup", post(routes::signup))
        .route("/login", post(routes::login))
        // Scrobbling
        .route("/now", post(routes::now_playing))
        .route("/scrob", post(routes::scrobble))
        // Stats
        .route("/recent", get(routes::recent_scrobbles))
        .route("/top/artists", get(routes::top_artists))
        .route("/top/tracks", get(routes::top_tracks))
        // Admin
        .route("/admin/users", get(routes::list_users))
        .route("/admin/users/{id}", get(routes::get_user))
        .route("/admin/users/{id}", axum::routing::delete(routes::delete_user))
        .route("/admin/users/{id}/admin", post(routes::toggle_admin))
        .route("/admin/stats", get(routes::get_stats))
        .route("/admin/scrobbles/{id}", axum::routing::delete(routes::delete_scrobble))
        // Health check
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    // Run server
    let listener = tokio::net::TcpListener::bind(&config.bind_address()).await?;
    tracing::info!("REST API: http://{}", config.bind_address());

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
