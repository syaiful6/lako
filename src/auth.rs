use std::env;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use log::error;
use jsonwebtoken::{encode, Header};
use serde_derive::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Claims {
    sub: i32,
    exp: u64,
}

impl Claims {
    pub fn new(user_id: i32, expire_in: u64) -> Claims {
        Claims {
            sub: user_id,
            exp: seconds_from_now(expire_in),
        }
    }

    pub fn user_id(&self) -> i32 {
        self.sub
    }
}

fn get_jwt_secret_key() -> String {
    match env::var("JWT_SECRET_KEY") {
        Ok(val)     => val,
        Err(e)      => {
            error!("Failed to get JWT_SECRET_KEY env: {}", e);
            "secret".to_string()
        }
    }
}

pub fn encode_token(sub: i32) -> String {
    encode(&Header::default(), &Claims::new(sub, 86400), get_jwt_secret_key().as_ref()).unwrap()
}

fn seconds_from_now(secs: u64) -> u64 {
    let expire_time =
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + Duration::from_secs(secs);
    
    expire_time.as_secs()
}