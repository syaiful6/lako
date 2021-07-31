use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::{self, insert_into};

use crate::models::user::User;
use crate::schema::companies;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[belongs_to(User)]
#[table_name = "companies"]
pub struct Company {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub address_1: String,
    pub address_2: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub country: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name = "companies"]
pub struct NewCompany {
    pub user_id: i32,
    pub name: String,
    pub address_1: String,
    pub address_2: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub country: String,
}

impl NewCompany {
    pub fn insert_company(&self, conn: &PgConnection) -> Result<Company, Error> {
        insert_into(crate::schema::companies::table)
            .values(self)
            .get_result(&*conn)
    }
}

pub fn delete_company(company_id: i32, owner_id: i32, conn: &PgConnection) -> Result<usize, Error> {
    use crate::schema::companies::dsl::*;
    use diesel::delete;

    delete(companies.find(company_id))
        .filter(user_id.eq(owner_id))
        .execute(&*conn)
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name = "companies"]
pub struct ChangeCompany {
    pub user_id: Option<i32>,
    pub name: Option<String>,
    pub address_1: Option<String>,
    pub address_2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub country: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl ChangeCompany {
    pub fn update(
        &self,
        owner_id: i32,
        company_id: i32,
        conn: &PgConnection,
    ) -> Result<Company, Error> {
        use crate::schema::companies::dsl::*;
        use diesel::update;

        update(companies.find(company_id))
            .filter(user_id.eq(owner_id))
            .set(self)
            .get_result::<Company>(&*conn)
    }
}

#[derive(Debug, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[table_name = "companies"]
pub struct CompactCompany {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
