use serde_derive::Deserialize;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct TokenPath {
    pub token: String,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct UserPath {
    pub id: i32,
}