use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;
use tracing::info;
use uuid::Uuid;

use crate::{
    audit::AuditEvent,
    models::{CreatePaymentRequest, PaymentResponse},
    AppState,
};

pub async fn create_payment(
    State(state): State<AppState>,
    Json(req): Json<CreatePaymentRequest>,
) -> Result<(StatusCode, Json<PaymentResponse>), (StatusCode, String)> {
    let payment_id = Uuid::new_v4();

    let merchant_id = match &req.merchant_id {
        Some(id) => {
            Uuid::parse_str(id).unwrap_or_else(|_| {
                Uuid::new_v5(&Uuid::NAMESPACE_DNS, id.as_bytes())
            })
        }
        None => Uuid::new_v4(),
    };

    let result = sqlx::query(
        r#"
        INSERT INTO payments (id, merchant_id, amount, currency, status)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(payment_id)
    .bind(merchant_id)
    .bind(req.amount)
    .bind(&req.currency)
    .bind("succeeded")
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            info!(
                "Payment created atomically: {} (event created by trigger)",
                payment_id
            );

            let _ = state.audit_tx.send(AuditEvent {
                merchant_id: Some(merchant_id),
                action: "payment_created".to_string(),
                details: json!({"payment_id": payment_id, "amount": req.amount}),
            }).await;

            Ok((
                StatusCode::CREATED,
                Json(PaymentResponse {
                    id: payment_id,
                    amount: req.amount,
                    currency: req.currency,
                    status: "succeeded".to_string(),
                }),
            ))
        }
        Err(e) => {
            tracing::error!("Failed to create payment: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create payment: {}", e),
            ))
        }
    }
}
