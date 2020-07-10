use chrono::NaiveDateTime;

use crate::models::user::User;
use crate::schema::emails;

#[derive(Debug, Queryable, AsChangeset, Identifiable, Associations)]
#[belongs_to(User)]
pub struct Email {
    pub id: i32,
    pub user_id: i32,
    pub email: String,
    pub token: String,
    pub verified: bool,
    pub token_generated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, AsChangeset)]
#[table_name = "emails"]
pub struct NewEmail<'a> {
    pub user_id: i32,
    pub email: &'a str,
}
