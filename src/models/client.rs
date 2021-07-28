use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::{self, insert_into};

use crate::models::user::User;
use crate::schema::clients;
use serde_derive::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[belongs_to(User)]
pub struct Client {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub email: String,
    pub company_name: String,
    pub address_1: String,
    pub address_2: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub country: String,
    pub website: String,
    pub notes: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub fn delete_client(client_id: i32, owner_id: i32, conn: &PgConnection) -> Result<usize, Error> {
    use crate::schema::clients::dsl::*;
    use diesel::delete;

    delete(clients.find(client_id))
        .filter(user_id.eq(owner_id))
        .execute(conn)
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name = "clients"]
pub struct NewClient {
    pub user_id: i32,
    pub name: String,
    pub email: String,
    pub company_name: String,
    pub address_1: String,
    pub address_2: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub country: String,
    pub website: String,
    pub notes: String,
}

impl NewClient {
    pub fn insert_client(self, conn: &PgConnection) -> Result<Client, Error> {
        let client = insert_into(crate::schema::clients::table)
            .values(&self)
            .get_result::<Client>(conn)?;

        Ok(client)
    }
}

#[derive(AsChangeset, Serialize, Deserialize, Validate)]
#[table_name = "clients"]
pub struct ChangeClient {
    pub user_id: Option<i32>,
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub company_name: Option<String>,
    pub address_1: Option<String>,
    pub address_2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub country: Option<String>,
    pub website: Option<String>,
    pub notes: Option<String>,
}

impl ChangeClient {
    pub fn update(
        self,
        owner_id: i32,
        client_id: i32,
        conn: &PgConnection,
    ) -> Result<Client, Error> {
        use crate::schema::clients::dsl::*;
        use diesel::update;

        let client = update(clients.find(client_id))
            .filter(user_id.eq(owner_id))
            .set(&self)
            .get_result::<Client>(conn)?;

        Ok(client)
    }
}

#[derive(Debug, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[table_name = "clients"]
pub struct CompactClient {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub company_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
