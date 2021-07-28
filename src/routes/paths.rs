use serde_derive::Deserialize;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct TokenPath {
    pub token: String,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct ResourceIDPath {
    pub id: i32,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct PaginationExtractor {
    pub per_page: Option<i64>,
    pub page: Option<i64>,
    pub q: Option<String>,
}
