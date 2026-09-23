use axum::{routing::{get, post}, Router};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::sync::mpsc;
use tracing::info;

pub mod audit;
pub mod models;
pub mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub audit_tx: mpsc::Sender<audit::AuditEvent>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(50) // Increased for load testing
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Initialize audit channel and worker
    let (audit_tx, audit_rx) = mpsc::channel(100);
    audit::spawn_audit_worker(pool.clone(), audit_rx);

    let state = AppState {
        db: pool,
        audit_tx,
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        .route("/keys/generate", post(routes::keys::generate_keys))
        .route("/payments", post(routes::payments::create_payment))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    info!("NEW ARCHITECTURE API listening on port {}", port);

    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}
