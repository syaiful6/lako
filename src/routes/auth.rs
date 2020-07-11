use futures::{future, Future};
use serde_derive::{Deserialize, Serialize};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::state::{FromState, State};
use gotham_middleware_jwt::AuthorizationToken;
use hyper::StatusCode;
use validator::Validate;

use crate::auth::{encode_token, Claims};
use crate::db::Repo;
use crate::routes::utils::{extract_json, json_response_ok, json_response_bad_message};
use crate::routes::paths::{TokenPath, UserPath};
use crate::models::user::{
    register_user, try_user_login, find_user, AuthenticationError, verify_email_with_token,
    regenerate_email_token_and_send
};
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


/// server POST /api/v1/register
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
                let res = json_response_ok(&state, &R { id: user.id });
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


/// serve POST /api/v1/login
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
                let res = json_response_ok(&state, &R { access: token });
                future::ok((state, res))
            } else {
                let res = json_response_bad_message(
                    &state,
                    "invalid username or password".into(),
                );
                future::ok((state, res))
            }
        });

    Box::new(f)
}

/// GET /api/v1/me
pub fn get_user(state: State) -> Box<HandlerFuture> {
    let repo = Repo::borrow_from(&state).clone();
    let token = AuthorizationToken::<Claims>::borrow_from(&state);
    let user_id = token.0.claims.user_id();

    let f = repo.run(move |conn| find_user(&conn, user_id))
        .map_err(|e| e.into_handler_error().with_status(StatusCode::BAD_REQUEST))
        .then(move |result| {
            if let Ok(Some(user)) = result {
                let res = json_response_ok(&state, &user);
                future::ok((state, res))
            } else {
                let res = json_response_bad_message(
                    &state,
                    "invalid token".into(),
                );
                future::ok((state, res))
            }
        });

    Box::new(f)
}

#[derive(Debug, Serialize)]
struct OkBool {
    ok: bool,
}

/// PUT /api/v1/confirm/:token
pub fn confirm_user_email(state: State) -> Box<HandlerFuture> {
    let token = {
        let path = TokenPath::borrow_from(&state).clone();
        path.token.to_owned()
    };
    let repo = Repo::borrow_from(&state).clone();
    
    let f = repo.run(move |conn| verify_email_with_token(&conn, token.as_str()))
        .map_err(|e| e.into_handler_error().with_status(StatusCode::BAD_REQUEST))
        .then(move |result| match result {
            Ok(b) => {
                let res = json_response_ok(&state, &OkBool{ ok: b });
                future::ok((state, res))
            }
            Err(_) => {
                let res = json_response_bad_message(
                    &state,
                    "Email belonging to token not found.".into(),
                );
                future::ok((state, res))
            }
        });

    Box::new(f)
}

/// Handles `PUT /user/:user_id/resend` route
pub fn regenerate_token_and_send(state: State) -> Box<HandlerFuture> {
    let user = UserPath::borrow_from(&state);
    // get current user id
    let token = AuthorizationToken::<Claims>::borrow_from(&state);
    let current_user_id = token.0.claims.user_id();

    if user.id != current_user_id {
        let res = json_response_bad_message(
            &state,
            "current user does not match requested user.".into()
        );
        
        return  Box::new(future::ok((state, res)))
    }

    let repo = Repo::borrow_from(&state).clone();

    let f = repo.run(move |conn| regenerate_email_token_and_send(&conn, current_user_id))
        .then(move |result| match result {
            Ok(b) => {
                let res = json_response_ok(&state, &OkBool{ok: b});
                future::ok((state, res))
            }
            Err(_) => {
                let res = json_response_bad_message(&state, "Email belonging to token not found.".into());
                future::ok((state, res))
            }
        });

    Box::new(f)
}
