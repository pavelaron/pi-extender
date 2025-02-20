use std::env;
use chrono::{Duration, Utc};
use hsh::{models::hash::Hash, random_string};
use jsonwebtoken::{
  encode,
  EncodingKey,
  Header,
  errors::Error,
};
use sled::Db;
use super::structs::Claims;

pub fn get_salt(data: &Db) -> String {
  let key = "keys::SALT";

  if data.contains_key(key).unwrap() {
    let stored = data.get(key).unwrap().unwrap();
    let value: &[u8] = stored.as_ref();

    return String::from_utf8(value.to_vec()).unwrap();
  }

  let salt = random_string!(10);

  data.insert(key, salt.as_str())
    .expect("Failed to save salt");

  let _ = data.flush();

  salt
}

pub fn hash_password(password: &str, salt: &String) -> String {
  let algo = "argon2i";

  Hash::new(password, salt.as_str(), algo)
    .expect("Failed to create hash")
    .to_string_representation()
}

pub fn generate_token(user_id: &str) -> Result<String, Error> {
  let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

  let expiration = Utc::now()
    .checked_add_signed(Duration::minutes(10))
    .expect("valid timestamp")
    .timestamp();

  let claims = Claims {
    sub: user_id.to_string(),
    exp: expiration,
    iat: Utc::now().timestamp(),
  };

  encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
  )
}
