mod auth;
mod config;
mod db;
mod graphql;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
  extract::State,
  http::{header, HeaderMap, StatusCode},
  response::{Html, IntoResponse},
  routing::{get, post},
  Router,
};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use db::DbPool;
use graphql::{build_schema, context::GraphQLContext, AppSchema};

#[derive(Clone)]
struct AppState {
  schema: AppSchema,
  pool: DbPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Load .env file if present
  let _ = dotenvy::dotenv();

  // Initialize tracing
  tracing_subscriber::registry()
    .with(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "scrob=debug,tower_http=debug".into()),
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

  // Build GraphQL schema
  let schema = build_schema();

  let state = AppState { schema, pool };

  // Build router
  let app = Router::new()
    .route("/graphql", post(graphql_handler))
    .route("/playground", get(graphql_playground))
    .route("/health", get(health_check))
    .layer(CorsLayer::permissive())
    .with_state(state);

  // Run server
  let listener = tokio::net::TcpListener::bind(&config.bind_address()).await?;
  tracing::info!("GraphQL endpoint: http://{}/graphql", config.bind_address());
  tracing::info!("GraphQL playground: http://{}/playground", config.bind_address());

  axum::serve(listener, app).await?;

  Ok(())
}

async fn graphql_handler(
  State(state): State<AppState>,
  headers: HeaderMap,
  req: GraphQLRequest,
) -> GraphQLResponse {
  // Extract user from Authorization header
  let current_user = if let Some(auth_value) = headers.get(header::AUTHORIZATION) {
    if let Ok(auth_str) = auth_value.to_str() {
      if let Some(token) = auth::extract_token_from_header(auth_str) {
        match auth::get_user_by_token(&state.pool, &token).await {
          Ok(user) => user,
          Err(e) => {
            tracing::error!("Error looking up user by token: {}", e);
            None
          }
        }
      } else {
        None
      }
    } else {
      None
    }
  } else {
    None
  };

  // Create GraphQL context
  let ctx = GraphQLContext::new(current_user);

  // Execute request
  state
    .schema
    .execute(req.into_inner().data(ctx).data(state.pool))
    .await
    .into()
}

async fn graphql_playground() -> impl IntoResponse {
  Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

async fn health_check() -> impl IntoResponse {
  (StatusCode::OK, "OK")
}
