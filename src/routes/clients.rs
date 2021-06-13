use futures::prelude::*;
use gotham::handler::HandlerFuture;
use gotham::helpers::http::response::create_empty_response;
use gotham::hyper::StatusCode;
use gotham::state::{FromState, State};
use gotham_middleware_jwt::AuthorizationToken;
use serde_derive::Deserialize;
use std::pin::Pin;
use validator::Validate;

use crate::auth::Claims;
use crate::db::Repo;
use crate::models::client::{delete_client, ChangeClient, NewClient};
use crate::routes::paths::ResourceIDPath;
use crate::routes::utils::{
    extract_json, json_response_bad_message, json_response_created, json_response_not_found,
    json_response_ok,
};

#[derive(Debug, Deserialize, Validate)]
struct NewClientRequest {
    pub name: String,
    #[validate(email)]
    pub email: String,
    pub company_name: String,
    pub address_1: String,
    pub address_2: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub country: String,
    pub website: String,
    pub notes: String,
}

/// serve POST /api/v1/clients
/// this route create a client for logged in user
pub fn create_client_handler(mut state: State) -> Pin<Box<HandlerFuture>> {
    let repo = Repo::borrow_from(&state).clone();

    let token = AuthorizationToken::<Claims>::borrow_from(&state);
    let current_user_id = token.0.claims.user_id();

    async move {
        let new_client = match extract_json::<NewClientRequest>(&mut state).await {
            Ok(client) => match client.validate() {
                Ok(_) => client,
                Err(e) => return Err((state, e.into())),
            },
            Err(e) => return Err((state, e)),
        };

        let result = repo
            .run(move |conn| {
                let new_client = NewClient {
                    user_id: current_user_id,
                    name: new_client.name,
                    email: new_client.email,
                    company_name: new_client.company_name,
                    address_1: new_client.address_1,
                    address_2: new_client.address_2,
                    city: new_client.city,
                    state: new_client.state,
                    zip_code: new_client.zip_code,
                    country: new_client.country,
                    notes: new_client.notes,
                    website: new_client.website,
                };
                new_client.insert_client(&conn)
            })
            .await;

        match result {
            Ok(client) => {
                let res = json_response_created(&state, &client);
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

/// serve PUT /api/v1/clients/:id
pub fn update_client_handler(mut state: State) -> Pin<Box<HandlerFuture>> {
    let token = AuthorizationToken::<Claims>::borrow_from(&state);
    let current_user_id = token.0.claims.user_id();

    let client_id = {
        let res = ResourceIDPath::borrow_from(&state);
        res.id
    };
    let repo = Repo::borrow_from(&state).clone();

    async move {
        let changes = match extract_json::<ChangeClient>(&mut state).await {
            Ok(changes) => changes,
            Err(e) => return Err((state, e)),
        };

        let result = repo
            .run(move |conn| changes.update(current_user_id, client_id, &conn))
            .await;

        match result {
            Ok(client) => {
                let res = json_response_ok(&state, &client);
                Ok((state, res))
            }
            Err(_) => {
                let res = json_response_bad_message(
                    &state,
                    "Unexpected error detected when trying to update client.".into(),
                );
                Ok((state, res))
            }
        }
    }
    .boxed()
}

/// serve DELETE /api/v1/clients/:id
pub fn delete_client_handler(state: State) -> Pin<Box<HandlerFuture>> {
    let token = AuthorizationToken::<Claims>::borrow_from(&state);
    let current_user_id = token.0.claims.user_id();

    let client_id = {
        let res = ResourceIDPath::borrow_from(&state);
        res.id
    };
    let repo = Repo::borrow_from(&state).clone();

    async move {
        let result = repo
            .run(move |conn| delete_client(client_id, current_user_id, &conn))
            .await;

        match result {
            Ok(deleted_count) => {
                if deleted_count > 0 {
                    let res = create_empty_response(&state, StatusCode::NO_CONTENT);
                    Ok((state, res))
                } else {
                    let res = json_response_not_found(&state, "That resource is not found".into());
                    Ok((state, res))
                }
            }
            Err(_) => {
                let res = json_response_bad_message(
                    &state,
                    "Unexpected error detected when trying to update client.".into(),
                );
                Ok((state, res))
            }
        }
    }
    .boxed()
}
