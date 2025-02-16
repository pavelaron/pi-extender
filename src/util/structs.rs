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

#[derive(Serialize)]
pub struct TokenResponse {
  token: String,
}
