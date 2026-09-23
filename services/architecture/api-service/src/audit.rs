use sqlx::PgPool;
use tokio::sync::mpsc;
use uuid::Uuid;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub merchant_id: Option<Uuid>,
    pub action: String,
    pub details: Value,
}

pub fn spawn_audit_worker(pool: PgPool, mut rx: mpsc::Receiver<AuditEvent>) {
    tokio::spawn(async move {
        tracing::info!("Audit worker started");
        while let Some(event) = rx.recv().await {
            match sqlx::query(
                "INSERT INTO audit_ledger (merchant_id, action, details) VALUES ($1, $2, $3)"
            )
            .bind(event.merchant_id)
            .bind(&event.action)
            .bind(&event.details)
            .execute(&pool)
            .await
            {
                Ok(_) => tracing::debug!("Audit event logged: {}", event.action),
                Err(e) => tracing::error!("Failed to log audit event: {}", e),
            }
        }
        tracing::info!("Audit worker shutting down");
    });
}
