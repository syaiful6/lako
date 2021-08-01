use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::{self, insert_into};
use rust_decimal::Decimal;

use crate::models::client::Client;
use crate::schema::invoices;
use crate::sqlx::invoice::{BillingReason, InvoiceStatus};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[belongs_to(Client)]
pub struct Invoice {
    pub id: i32,
    pub user_id: i32,
    pub client_id: i32,
    pub company_id: i32,
    pub invoice_number: String,
    pub description: String,
    pub currency: String,
    pub status: InvoiceStatus,
    pub billing_reason: BillingReason,
    pub due_date: Option<NaiveDateTime>,
    pub invoice_date: Option<NaiveDateTime>,
    pub last_send_date: Option<NaiveDateTime>,
    pub amount: Decimal,
    pub balance: Decimal,
    pub discount: Decimal,
    pub tax: Decimal,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name = "invoices"]
pub struct NewInvoice {
    pub user_id: i32,
    pub client_id: i32,
    pub company_id: i32,
    pub invoice_number: String,
    pub description: String,
    pub currency: String,
    pub status: InvoiceStatus,
    pub billing_reason: BillingReason,
    pub due_date: Option<NaiveDateTime>,
    pub invoice_date: Option<NaiveDateTime>,
    pub last_send_date: Option<NaiveDateTime>,
    pub amount: Decimal,
    pub balance: Decimal,
    pub discount: Decimal,
    pub tax: Decimal,
}

impl Invoice {
    pub fn insert(invoice: &NewInvoice, conn: &PgConnection) -> Result<Invoice, Error> {
        insert_into(crate::schema::invoices::table)
            .values(invoice)
            .get_result(conn)
    }
}
