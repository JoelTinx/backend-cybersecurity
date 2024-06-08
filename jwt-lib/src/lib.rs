use chrono::{Duration, Utc};
use jsonwebtoken::{decode, DecodingKey, encode, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
  email: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
  email: String,
  exp: i64
}

pub fn get_jwt(user: User) -> Result<String, String> {
  let token = encode(
    &Header::default(),
    &Claims {
      email: user.email,
      exp: (Utc::now() + Duration::minutes(5)).timestamp()
    },
    &EncodingKey::from_secret("my_secret".as_bytes())
  )
    .map_err(|e| e.to_string());

  token
}


pub fn decode_jwt(token: &str) -> Result<User, String> {
  let token_data = decode::<User>(
    token,
    &DecodingKey::from_secret("my_secret".as_bytes()),
    &Validation::default()
  );
  
  match token_data {
    Ok(token_data) => Ok(token_data.claims),
    Err(e) => Err(e.to_string()),
  }
}