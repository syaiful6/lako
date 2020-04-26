use futures::{future, Future};
use serde_derive::{Deserialize, Serialize};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::state::{FromState, State};
use gotham::helpers::http::response::create_response;
use hyper::StatusCode;
use validator::Validate;

use crate::auth::encode_token;
use crate::db::Repo;
use crate::routes::utils::extract_json;
use crate::models::user::{register_user, try_user_login, AuthenticationError};
use crate::sql_types::Role;

#[derive(Debug, Deserialize, Validate)]
struct NewUser {
    #[validate(length(min = 5))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 8))]
    password1: String,
    #[validate(length(min = 8))]
    password2: String,    
}

/// server /private/register
/// this route register user as customer
pub fn register_user_handler(mut state: State) -> Box<HandlerFuture> {
    let repo = Repo::borrow_from(&state).clone();

    #[derive(Serialize)]
    struct R {
        id: i32,
    }
    let f = extract_json::<NewUser>(&mut state)
        .and_then(move |user| match user.validate() {
            Ok(_) => {
                if user.password1 == user.password2 {
                    future::ok(user)
                } else {
                    future::err(
                        AuthenticationError::IncorrectPassword
                            .into_handler_error()
                            .with_status(StatusCode::BAD_REQUEST)
                    )
                }
            }
            Err(e) => future::err(
                e.into_handler_error().with_status(StatusCode::BAD_REQUEST)
            )
        })
        .and_then(move |user| {
            repo.run(move |conn| {
                register_user(
                    &conn,
                    user.username.to_ascii_lowercase().as_str(),
                    user.email.to_ascii_lowercase().as_str(),
                    user.password1.as_str(),
                    &Role::Customer,
                )
            }).map_err(|e| e.into_handler_error().with_status(StatusCode::BAD_REQUEST))
        })
        .then(|result| match result {
            Ok(user) => {
                let body = serde_json::to_string(&R { id: user.id })
                    .expect("Failed to serialize users");
                let res = create_response(&state, StatusCode::CREATED, mime::APPLICATION_JSON, body);
                future::ok((state, res))
            }
            Err(e) => future::err((state, e))
        });

    Box::new(f)
}

#[derive(Debug, Deserialize, Validate)]
struct LoginForm {
    #[validate(length(min = 1))]
    username: String,
    #[validate(length(min = 8))]
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginErr {
    message: String,
}

/// serve /private/login
pub fn login_user_handler(mut state: State) -> Box<HandlerFuture> {
    let repo = Repo::borrow_from(&state).clone();

    #[derive(Serialize)]
    struct R {
        access: String,
    }

    let f = extract_json::<LoginForm>(&mut state)
        .and_then(move |creds| match creds.validate() {
            Ok(_) => future::ok(creds),
            Err(e) => future::err(e.into_handler_error().with_status(StatusCode::BAD_REQUEST)),
        })
        .and_then(move |creds| {
            repo.run(move |conn| {
                try_user_login(
                    &conn,
                    creds.username.to_ascii_lowercase().as_str(),
                    creds.password.as_str(),
                )
            }).map_err(|e| e.into_handler_error().with_status(StatusCode::BAD_REQUEST))
        })
        .then(|result| {
            if let Ok(Some(user)) = result {
                let token = encode_token(user.id);
                let body = serde_json::to_string(&R { access: token })
                    .expect("Failed to serialize token");
                let res = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, body);
                future::ok((state, res))
            } else {
                let body = serde_json::to_string(&LoginErr{ message: "invalid username or password".into() })
                        .expect("Failed to serialize error");
                let res = create_response(&state, StatusCode::BAD_REQUEST, mime::APPLICATION_JSON, body);
                future::ok((state, res))
            }
        });

    Box::new(f)
}