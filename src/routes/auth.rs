use futures::{future, Future};
use serde_derive::{Deserialize, Serialize};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::state::{FromState, State};
use gotham::helpers::http::response::create_response;
use hyper::StatusCode;

use crate::db::Repo;
use crate::routes::utils::extract_json;
use crate::models::user::register_user;

#[derive(Deserialize)]
struct NewUser {
    username: String,
    email: String,
    password: String,    
}

pub fn register_user_handler(mut state: State) -> Box<HandlerFuture> {
    let repo = Repo::borrow_from(&state).clone();

    #[derive(Serialize)]
    struct R {
        id: i32,
    }
    let f = extract_json::<NewUser>(&mut state)
        .and_then(move |user| {
            repo.run(move |conn| {
                register_user(
                    &conn,
                    user.username.as_str(),
                    user.email.as_str(),
                    user.password.as_str(),
                )
            }).map_err(|e| e.into_handler_error())
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