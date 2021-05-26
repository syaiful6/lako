use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::{self, insert_into};

use crate::models::user::User;
use crate::schema::clients;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[belongs_to(User)]
pub struct Client {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub email: String,
    pub company_name: String,
    pub phone_number: String,
    pub company_website: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "clients"]
pub struct NewClient {
    pub user_id: i32,
    pub name: String,
    pub email: String,
    pub company_name: String,
    pub phone_number: String,
    pub company_website: String,
}

/// insert a client
pub fn insert_client(conn: &PgConnection, new_client: &NewClient) -> Result<Client, Error> {
    let client = insert_into(crate::schema::clients::table)
        .values(new_client)
        .get_result::<Client>(conn)?;

    Ok(client)
}
