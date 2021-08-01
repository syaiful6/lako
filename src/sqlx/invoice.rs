use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Smallint;
use diesel::*;
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::io::Write;

#[derive(AsExpression, FromSqlRow, PartialEq, Eq, Debug, Clone)]
#[sql_type = "Smallint"]
pub enum InvoiceStatus {
    Draft,
    Open,
    Paid,
    Uncollectible,
    Void,
}

#[derive(AsExpression, FromSqlRow, PartialEq, Eq, Debug, Clone)]
#[sql_type = "Smallint"]
pub enum BillingReason {
    Manual,
    ContractCycle,
}

impl ToSql<Smallint, Pg> for InvoiceStatus {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let t = match *self {
            InvoiceStatus::Draft => 0,
            InvoiceStatus::Open => 1,
            InvoiceStatus::Paid => 2,
            InvoiceStatus::Uncollectible => 3,
            InvoiceStatus::Void => 4,
        };
        <i16 as ToSql<Smallint, Pg>>::to_sql(&t, out)
    }
}

impl ToSql<Smallint, Pg> for BillingReason {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let t = match *self {
            BillingReason::Manual => 0,
            BillingReason::ContractCycle => 1,
        };
        <i16 as ToSql<Smallint, Pg>>::to_sql(&t, out)
    }
}

impl FromSql<Smallint, Pg> for InvoiceStatus {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match <i16 as FromSql<Smallint, Pg>>::from_sql(bytes)? {
            0 => Ok(InvoiceStatus::Draft),
            1 => Ok(InvoiceStatus::Open),
            2 => Ok(InvoiceStatus::Paid),
            3 => Ok(InvoiceStatus::Uncollectible),
            4 => Ok(InvoiceStatus::Void),
            _ => Err("Unrecognized enum variant of InvoiceStatus".into()),
        }
    }
}

impl FromSql<Smallint, Pg> for BillingReason {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match <i16 as FromSql<Smallint, Pg>>::from_sql(bytes)? {
            0 => Ok(BillingReason::Manual),
            1 => Ok(BillingReason::ContractCycle),
            _ => Err("Unrecognized enum variant of billing reason".into()),
        }
    }
}

impl Serialize for InvoiceStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match *self {
            InvoiceStatus::Draft => "draft",
            InvoiceStatus::Open => "open",
            InvoiceStatus::Paid => "paid",
            InvoiceStatus::Uncollectible => "uncollectible",
            InvoiceStatus::Void => "void",
        })
    }
}

impl<'de> Deserialize<'de> for InvoiceStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "draft" => Ok(InvoiceStatus::Draft),
            "open" => Ok(InvoiceStatus::Open),
            "paid" => Ok(InvoiceStatus::Paid),
            "uncollectible" => Ok(InvoiceStatus::Uncollectible),
            "void" => Ok(InvoiceStatus::Void),
            e => Err(serde::de::Error::custom(format!(
                "Failed to deserialize invoice status: {}",
                e
            ))),
        }
    }
}

impl Serialize for BillingReason {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match *self {
            BillingReason::Manual => "manual",
            BillingReason::ContractCycle => "contract_cycle",
        })
    }
}

impl<'de> Deserialize<'de> for BillingReason {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "manual" => Ok(BillingReason::Manual),
            "contract_cycle" => Ok(BillingReason::ContractCycle),
            e => Err(serde::de::Error::custom(format!(
                "Failed to deserialize billing reason: {}",
                e
            ))),
        }
    }
}
