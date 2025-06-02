use chrono::{Duration, Utc};
use dotenv::dotenv;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    // aud: String,         // Optional. Audience
    exp: usize,          // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize,          // Optional. Issued at (as UTC timestamp)
    iss: String,         // Optional. Issuer
    // nbf: usize,          // Optional. Not Before (as UTC timestamp)
    sub: String,         // Optional. Subject (whom token refers to)
}

fn create_jwt_claims(user_id: String) -> Claims {
    let now = Utc::now();
    let claim = Claims {
        // aud: "http://127.0.0.1:3000",
        exp: now + Duration::minutes(15), // expires in 15 mins
        iat: now,
        iss: "http://127.0.0.1:8080",
        // nbf: now,
        sub: user_id,
    };

    claim
}

fn encode_jwt_token(user_id: String) -> Result<String, Error> {
    dotenv().ok();

    let jwt_secret =
        env::var("JWT_SECRET").expect("ERROR: JWT_SECRET must be present in '.env'");

    let claims = create_jwt_claims(user_id);

    match encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_ref()))? {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{:?}: Failed to encode JWT token", Utc::now());
            e
        }
    }
}

fn decode_jwt_token(token: String) -> Result<String, Error> {
    dotenv().ok();

    let jwt_secret =
        env::var("JWT_SECRET").expect("ERROR: JWT_SECRET must be present in '.env'");

    match decode::<Claims>(&token, &DecodingKey::from_secret(jwt_secret.as_ref()), &Validation::default())? {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{:?}: Failed to decode JWT token", Utc::now());
            e
        }
    }
}