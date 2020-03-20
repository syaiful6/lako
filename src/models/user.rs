use bcrypt::*;
use diesel::prelude::*;
use diesel::{
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    sql_types::SmallInt,
};
use diesel::{self, insert_into};

use crate::schema::{emails, users};

#[derive(Debug, Clone, Copy, PartialEq, FromSqlRow, AsExpression)]
#[repr(i32)]
#[sql_type = "SmallInt"]
pub enum Role {
    Customer  = 0,
    Staff     = 1,
    SuperUser = 2,
}

impl Into<&'static str> for Role {
    fn into(self) -> &'static str {
        match self {
            Role::Customer  => "customer",
            Role::Staff     => "staff",
            Role::SuperUser => "superuser"
        }
    }
}

impl FromSql<SmallInt, Pg> for Role {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match <i32 as FromSql<SmallInt, Pg>>::from_sql(bytes)? {
            0 => Ok(Role::Customer),
            1 => Ok(Role::Staff),
            2 => Ok(Role::SuperUser),
            n => Err(format!("unknown role: {}", n).into()),
        }
    }
}

impl ToSql<SmallInt, Pg> for Role {
    fn to_sql<W: Write>(&self, out: &mut Output<'_, W, Pg>) -> serialize::Result {
        ToSql::<SmallInt, Pg>::to_sql(&(*self as i16), out)
    }
}

#[derive(Debug)]
pub enum AuthError {
    IncorrectPassword,
    NoUsernameSet,
    NoPassword,
    BcryptError(BcryptError),
    DatabaseError(diesel::result::Error),
}

impl From<BcryptError> for AuthError {
    fn from(e: BcryptError) -> Self {
        AuthError::BcryptError(e)
    }
}

#[derive(Queryable, Identifiable, Debug, PartialEq)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub name: String,
    pub role: Role,
    pub image: String,
}

#[derive(Queryable)]
pub struct UserWithPassword {
    user: User,
    password: String,
}