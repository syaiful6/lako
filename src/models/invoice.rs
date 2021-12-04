use chrono::{Datelike, NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_types::{Int4, Timestamp, VarChar};
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

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name = "invoices"]
pub struct ChangeInvoice {
    pub user_id: Option<i32>,
    pub client_id: Option<i32>,
    pub company_id: Option<i32>,
    pub invoice_number: Option<String>,
    pub description: Option<String>,
    pub currency: Option<String>,
    pub status: Option<InvoiceStatus>,
    pub billing_reason: Option<BillingReason>,
    pub due_date: Option<NaiveDateTime>,
    pub invoice_date: Option<NaiveDateTime>,
    pub amount: Option<Decimal>,
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

sql_function! {
    fn date_part(x: VarChar, y: Timestamp) -> Int4;
}

impl Invoice {
    /// insert new invoice, the amount of invoice should already calculated correctly
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

    /// Get next invoice number for the given user_id and client_id
    pub fn get_next_invoice_number(
        user_id: i32,
        client_id: i32,
        conn: &PgConnection,
    ) -> Result<String, Error> {
        let current_year = Utc::now().naive_utc().year();
        let count = invoices::table
            .select(diesel::dsl::count(invoices::id))
            .filter(invoices::user_id.eq(user_id))
            .filter(invoices::client_id.eq(client_id))
            .filter(date_part("year", invoices::created_at).eq(current_year))
            .get_result::<i64>(conn)?;

        Ok(format!("{}/{:03}", current_year, count + 1))
    }

    pub fn update(
        primary_id: i32,
        owner_id: i32,
        changes: &ChangeInvoice,
        conn: &PgConnection,
    ) -> Result<Invoice, Error> {
        use crate::schema::invoices::dsl::*;
        use diesel::update;

        update(invoices.find(primary_id))
            .filter(user_id.eq(owner_id))
            .set(changes)
            .returning((
                id,
                invoice_id,
                user_id,
                client_id,
                company_id,
                invoice_number,
                description,
                currency,
                status,
                billing_reason,
                due_date,
                invoice_date,
                last_send_date,
                amount,
                balance,
                discount,
                tax,
                created_at,
                updated_at,
            ))
            .get_result::<Invoice>(conn)
    }

    pub fn delete(primary_id: i32, owner_id: i32, conn: &PgConnection) -> Result<usize, Error> {
        use crate::schema::invoices::dsl::*;
        use diesel::delete;

        delete(invoices.find(primary_id))
            .filter(user_id.eq(owner_id))
            .execute(&*conn)
    }

    /// Recalculate invoice amount
    pub fn recalculate_amount(invoice_id: i32, conn: &PgConnection) -> Result<Decimal, Error> {
        conn.transaction(|| {
            use crate::schema::invoices::dsl::{amount, invoices};
            use diesel::update;

            let items = invoice_items::table
                .select((invoice_items::amount, invoice_items::quantity))
                .filter(invoice_items::invoice_id.eq(invoice_id))
                .get_results::<(Decimal, Decimal)>(conn)?;

            let item_total = items.into_iter().map(|item| item.0 * item.1).sum();

            update(invoices.find(invoice_id))
                .set(amount.eq(item_total))
                .execute(conn)?;

            Ok(item_total)
        })
    }
}
