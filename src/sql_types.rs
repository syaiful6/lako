use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::sql_types::Smallint;
use diesel::serialize::{self, Output, ToSql};
use diesel::*;
use std::io::Write;

#[derive(AsExpression, FromSqlRow, PartialEq, Eq, Debug, Clone)]
#[sql_type = "Smallint"]
pub enum Role {
    Superuser,
    Staff,
    Customer,
}

impl ToSql<Smallint, Pg> for Role {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let t = match *self {
            Role::Superuser => 0,
            Role::Staff     => 1,
            Role::Customer  => 1,
        };
        <i16 as ToSql<Smallint, Pg>>::to_sql(&t, out)
    }
}

impl FromSql<Smallint, Pg> for Role {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match <i16 as FromSql<Smallint, Pg>>::from_sql(bytes)? {
            0 => Ok(Role::Superuser),
            1 => Ok(Role::Staff),
            2 => Ok(Role::Customer),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}