use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::sql_types::Smallint;
use diesel::serialize::{self, Output, ToSql};
use diesel::*;
use serde::de::{Deserializer, Deserialize};
use serde::ser::{Serializer, Serialize};
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
            Role::Customer  => 2,
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

impl Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(match *self {
            Role::Superuser => "superuser",
            Role::Staff     => "staff",
            Role::Customer  => "customer",
        })
    }
}

impl<'de> Deserialize<'de> for Role {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "superuser" => Ok(Role::Superuser),
            "staff"     => Ok(Role::Staff),
            "customer"  => Ok(Role::Customer),
            e           => Err(serde::de::Error::custom(format!(
                "Failed to deserialize role: {}",
                e
            ))),
        }
    }
}