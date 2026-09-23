use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Merchant {
    pub id: Uuid,
    pub username: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MerchantKey {
    pub merchant_id: Uuid,
    pub public_key: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payment {
    pub id: Uuid,
    pub amount: i64,
    pub currency: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreatePaymentRequest {
    pub merchant_id: Option<String>, // Keep Option for backwards compatibility with the demo
    pub amount: i64,
    pub currency: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaymentResponse {
    pub id: Uuid,
    pub amount: i64,
    pub currency: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthResponse {
    pub merchant_id: Uuid,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyGenerationResponse {
    pub merchant_id: Uuid,
    pub public_key: String,
}
