use axum::{extract::State, http::StatusCode, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde_json::json;
use uuid::Uuid;

use crate::{
    audit::AuditEvent,
    models::{AuthResponse, LoginRequest, RegisterRequest},
    AppState,
};

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, String)> {
    let hashed_pw = hash(&req.password, DEFAULT_COST).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Password hashing failed: {}", e),
        )
    })?;

    let merchant_id = Uuid::new_v4();

    let result = sqlx::query(
        "INSERT INTO merchants (id, username, password_hash) VALUES ($1, $2, $3)",
    )
    .bind(merchant_id)
    .bind(&req.username)
    .bind(&hashed_pw)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            // Send audit event
            let _ = state.audit_tx.send(AuditEvent {
                merchant_id: Some(merchant_id),
                action: "user_registered".to_string(),
                details: json!({"username": req.username}),
            }).await;

            Ok((
                StatusCode::CREATED,
                Json(AuthResponse {
                    merchant_id,
                    username: req.username,
                }),
            ))
        }
        Err(e) => {
            tracing::error!("Failed to register user: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                "Username might already exist".to_string(),
            ))
        }
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, String)> {
    let user_record = sqlx::query_as::<_, (Uuid, String, String)>(
        "SELECT id, username, password_hash FROM merchants WHERE username = $1"
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some((id, username, password_hash)) = user_record {
        if verify(&req.password, &password_hash).unwrap_or(false) {
            let _ = state.audit_tx.send(AuditEvent {
                merchant_id: Some(id),
                action: "user_login".to_string(),
                details: json!({}),
            }).await;

            return Ok((
                StatusCode::OK,
                Json(AuthResponse {
                    merchant_id: id,
                    username: username,
                }),
            ));
        }
    }

    Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))
}
