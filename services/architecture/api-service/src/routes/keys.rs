use axum::{extract::State, http::StatusCode, Json};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde_json::json;
use uuid::Uuid;

use crate::{
    audit::AuditEvent,
    models::KeyGenerationResponse,
    AppState,
};

#[derive(serde::Deserialize)]
pub struct KeyGenRequest {
    pub merchant_id: Uuid,
}

pub async fn generate_keys(
    State(state): State<AppState>,
    Json(req): Json<KeyGenRequest>,
) -> Result<(StatusCode, Json<KeyGenerationResponse>), (StatusCode, String)> {
    
    // Generate Ed25519 Keypair
    let mut csprng = OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let verifying_key: VerifyingKey = (&signing_key).into();

    use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
    
    // Encode to base64 for storage
    let priv_b64 = b64.encode(signing_key.to_bytes());
    let pub_b64 = b64.encode(verifying_key.to_bytes());

    // UPSERT into merchant_keys
    let result = sqlx::query(
        r#"
        INSERT INTO merchant_keys (merchant_id, public_key, private_key)
        VALUES ($1, $2, $3)
        ON CONFLICT (merchant_id) 
        DO UPDATE SET public_key = $2, private_key = $3, created_at = NOW()
        "#,
    )
    .bind(req.merchant_id)
    .bind(&pub_b64)
    .bind(&priv_b64)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            let _ = state.audit_tx.send(AuditEvent {
                merchant_id: Some(req.merchant_id),
                action: "keys_generated".to_string(),
                details: json!({"public_key": pub_b64}),
            }).await;

            Ok((
                StatusCode::CREATED,
                Json(KeyGenerationResponse {
                    merchant_id: req.merchant_id,
                    public_key: pub_b64,
                }),
            ))
        }
        Err(e) => {
            tracing::error!("Failed to save keys: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate keys".to_string(),
            ))
        }
    }
}
