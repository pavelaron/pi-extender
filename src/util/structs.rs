use serde::{Deserialize, Serialize};

// Claims structure for JWT payload
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub sub: String,  // Subject (user id)
  pub exp: i64,     // Expiration time
  pub iat: i64,     // Issued at
}

// Auth guard for protected routes
pub struct AuthenticatedUser {
  pub user_id: String,
}

#[derive(FromForm)]
pub struct LoginInput {
  pub username: String,
  pub password: String,
}

#[derive(FromForm)]
pub struct WirelessInput {
  pub source_ssid: String,
  pub source_password: String,
  pub ap_ssid: String,
  pub ap_password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
  token: String,
}
