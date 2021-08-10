use chrono::NaiveDateTime;
use futures::prelude::*;
use gotham::handler::HandlerFuture;
use gotham::state::{FromState, State};
use gotham_middleware_jwt::AuthorizationToken;
use rust_decimal::Decimal;
use serde_derive::{Deserialize, Serialize};
use std::pin::Pin;
use uuid::Uuid;
use validator::Validate;

use crate::auth::Claims;
use crate::db::Repo;
use crate::models::invoice::{Invoice, InvoiceItem, NewInvoice, NewInvoiceItem};
use crate::routes::utils::{extract_json, json_response_bad_message, json_response_created};
use crate::sqlx::invoice::{BillingReason, InvoiceStatus};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateInvoiceRequest {
    pub client_id: i32,
    pub company_id: i32,
    pub invoice_number: Option<String>,
    pub description: String,
    pub currency: String,
    pub status: InvoiceStatus,
    pub billing_reason: BillingReason,
    pub due_date: Option<NaiveDateTime>,
    pub invoice_date: Option<NaiveDateTime>,
    pub balance: Option<Decimal>,
    pub discount: Option<Decimal>,
    pub tax: Option<Decimal>,
    pub items: Vec<NewInvoiceItem>,
}

/// calculate the total amount
fn calculate_invoice_items_total(items: &Vec<NewInvoiceItem>) -> Decimal {
    items
        .into_iter()
        .map(|item| item.amount * item.quantity)
        .sum()
}

/// serve POST /api/v1/invoices
pub fn create_invoice_handler(mut state: State) -> Pin<Box<HandlerFuture>> {
    let repo = Repo::borrow_from(&state).clone();

    let token = AuthorizationToken::<Claims>::borrow_from(&state);
    let current_user_id = token.0.claims.user_id();

    async move {
        let create_invoice_req = match extract_json::<CreateInvoiceRequest>(&mut state).await {
            Ok(create_invoice_req) => create_invoice_req,
            Err(_) => {
                let res =
                    json_response_bad_message(&state, "Invalid create request payload".into());
                return Ok((state, res));
            }
        };

        let result = repo
            .run(move |conn| {
                let invoice_amount = calculate_invoice_items_total(&create_invoice_req.items);
                let invoice_number = match create_invoice_req.invoice_number {
                    Some(invoice_number) => invoice_number,
                    None => {
                        // TODO: generate invoice number
                        "2021/04".to_owned()
                    }
                };
                let new_invoice = NewInvoice {
                    invoice_id: Uuid::new_v4(),
                    user_id: current_user_id,
                    client_id: create_invoice_req.client_id,
                    company_id: create_invoice_req.company_id,
                    invoice_number: invoice_number,
                    description: create_invoice_req.description,
                    currency: create_invoice_req.currency,
                    status: create_invoice_req.status,
                    billing_reason: create_invoice_req.billing_reason,
                    due_date: create_invoice_req.due_date,
                    invoice_date: create_invoice_req.invoice_date,
                    amount: invoice_amount,
                    balance: create_invoice_req.balance,
                    discount: create_invoice_req.discount,
                    tax: create_invoice_req.tax,
                };
                Invoice::insert(&new_invoice, create_invoice_req.items, &conn)
            })
            .await;

        match result {
            Ok((invoice, items)) => {
                #[derive(Serialize)]
                struct R {
                    invoice: Invoice,
                    items: Vec<InvoiceItem>,
                }
                let res = json_response_created(&state, &R { invoice, items });
                Ok((state, res))
            }
            Err(_) => {
                let res = json_response_bad_message(
                    &state,
                    "Unexpected error when trying to insert invoices".into(),
                );
                Ok((state, res))
            }
        }
    }
    .boxed()
}
