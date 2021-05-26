use futures::prelude::*;
use gotham::handler::HandlerFuture;
use gotham::state::{FromState, State};
use gotham_middleware_jwt::AuthorizationToken;
use serde_derive::Deserialize;
use std::pin::Pin;
use validator::Validate;

use crate::auth::Claims;
use crate::db::Repo;
use crate::models::client::{insert_client, NewClient};
use crate::routes::utils::{extract_json, json_response_bad_message, json_response_ok};

#[derive(Debug, Deserialize, Validate)]
struct NewClientRequest {
    name: String,
    #[validate(email)]
    email: String,
    company_name: String,
    phone_number: String,
    company_website: String,
}

/// serve POST /api/v1/clients
/// this route create a client for logged in user
pub fn create_client_handler(mut state: State) -> Pin<Box<HandlerFuture>> {
    let repo = Repo::borrow_from(&state).clone();

    let token = AuthorizationToken::<Claims>::borrow_from(&state);
    let current_user_id = token.0.claims.user_id();

    async move {
        let new_client = match extract_json::<NewClientRequest>(&mut state).await {
            Ok(client) => client,
            Err(e) => return Err((state, e)),
        };

        let result = repo
            .run(move |conn| {
                insert_client(
                    &conn,
                    &NewClient {
                        user_id: current_user_id,
                        name: new_client.name,
                        email: new_client.email,
                        company_name: new_client.company_name,
                        phone_number: new_client.phone_number,
                        company_website: new_client.company_website,
                    },
                )
            })
            .await;

        match result {
            Ok(client) => {
                let res = json_response_ok(&state, &client);
                Ok((state, res))
            }
            Err(_) => {
                let res = json_response_bad_message(
                    &state,
                    "Unexpected error detected when trying to insert client.".into(),
                );
                Ok((state, res))
            }
        }
    }
    .boxed()
}
