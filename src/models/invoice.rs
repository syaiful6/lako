use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::{self, insert_into};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::models::client::Client;
use crate::schema::{invoice_items, invoices};
use crate::sqlx::invoice::{BillingReason, InvoiceStatus};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[belongs_to(Client)]
pub struct Invoice {
    pub id: i32,
    pub invoice_id: Uuid,
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
    pub invoice_id: Uuid,
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
    pub amount: Decimal,
    pub balance: Option<Decimal>,
    pub discount: Option<Decimal>,
    pub tax: Option<Decimal>,
}

#[derive(Debug, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[table_name = "invoice_items"]
pub struct InvoiceItem {
    pub id: i32,
    pub invoice_id: i32,
    pub name: String,
    pub description: String,
    pub amount: Decimal,
    pub quantity: Decimal,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name = "invoice_items"]
pub struct NewInvoiceItem {
    pub invoice_id: i32,
    pub name: String,
    pub description: String,
    pub amount: Decimal,
    pub quantity: Decimal,
}

impl Invoice {
    // insert new invoice, the amount of invoice should already calculated correctly
    pub fn insert(
        invoice: &NewInvoice,
        items: Vec<NewInvoiceItem>,
        conn: &PgConnection,
    ) -> Result<(Invoice, Vec<InvoiceItem>), Error> {
        conn.transaction(|| {
            let invoice = insert_into(crate::schema::invoices::table)
                .values(invoice)
                .get_result::<Invoice>(conn)?;

            if !Vec::is_empty(&items) {
                let insertable_items: Vec<_> = items
                    .into_iter()
                    .map(|new_item| NewInvoiceItem {
                        invoice_id: invoice.id,
                        ..new_item
                    })
                    .collect();

                let inserted_items = insert_into(crate::schema::invoice_items::table)
                    .values(insertable_items)
                    .get_results::<InvoiceItem>(conn)?;

                Ok((invoice, inserted_items))
            } else {
                Ok((invoice, Vec::new()))
            }
        })
    }
}
