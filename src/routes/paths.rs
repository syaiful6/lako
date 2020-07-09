use serde_derive::Deserialize;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct TokenPath {
    pub token: String,
}