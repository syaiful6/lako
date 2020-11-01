use futures::future;
use futures::prelude::*;
use gotham::handler::HandlerFuture;
use gotham::state::{FromState, State};
use gotham_middleware_jwt::AuthorizationToken;
use serde_derive::{Deserialize, Serialize};
use std::pin::Pin;
use validator::Validate;

use crate::auth::{encode_token, Claims};
use crate::db::Repo;
use crate::models::user::{
    find_user, regenerate_email_token_and_send, register_user, try_user_login,
    verify_email_with_token, AuthenticationError,
};
use crate::routes::paths::{TokenPath, UserPath};
use crate::routes::utils::{extract_json, json_response_bad_message, json_response_ok};
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

/// serve POST /api/v1/register
/// this route register user as customer
pub fn register_user_handler(mut state: State) -> Pin<Box<HandlerFuture>> {
    let repo = Repo::borrow_from(&state).clone();

    #[derive(Serialize)]
    struct R {
        id: i32,
    }

    async move {
        let user = match extract_json::<NewUser>(&mut state).await {
            Ok(user) => {
                if user.password1 == user.password2 {
                    user
                } else {
                    return Err((state, AuthenticationError::IncorrectPassword.into()));
                }
            }
            Err(e) => return Err((state, e)),
        };

        let result = repo
            .run(move |conn| {
                register_user(
                    &conn,
                    user.username.to_ascii_lowercase().as_str(),
                    user.email.to_ascii_lowercase().as_str(),
                    user.password1.as_str(),
                    &Role::Customer,
                )
            })
            .await;

        match result {
            Ok(user) => {
                let res = json_response_ok(&state, &R { id: user.id });

                Ok((state, res))
            }
            Err(e) => Err((state, e.into())),
        }
    }
    .boxed()
}

#[derive(Debug, Deserialize, Validate)]
struct LoginForm {
    #[validate(length(min = 1))]
    username: String,
    #[validate(length(min = 8))]
    password: String,
}

/// serve POST /api/v1/login
pub fn login_user_handler(mut state: State) -> Pin<Box<HandlerFuture>> {
    let repo = Repo::borrow_from(&state).clone();

    #[derive(Serialize)]
    struct R {
        access: String,
    }

    async move {
        let creds = match extract_json::<LoginForm>(&mut state).await {
            Ok(creds) => creds,
            Err(e) => return Err((state, e)),
        };

        let result = repo
            .run(move |conn| {
                try_user_login(
                    &conn,
                    creds.username.to_ascii_lowercase().as_str(),
                    creds.password.as_str(),
                )
            })
            .await;

        if let Ok(Some(user)) = result {
            let token = encode_token(user.id);
            let res = json_response_ok(&state, &R { access: token });

            Ok((state, res))
        } else {
            let res = json_response_bad_message(&state, "invalid username or password".into());
            Ok((state, res))
        }
    }
    .boxed()
}

/// GET /api/v1/me
pub fn get_user(state: State) -> Pin<Box<HandlerFuture>> {
    let repo = Repo::borrow_from(&state).clone();
    let token = AuthorizationToken::<Claims>::borrow_from(&state);
    let user_id = token.0.claims.user_id();

    async move {
        let result = repo.run(move |conn| find_user(&conn, user_id)).await;

        if let Ok(Some(user)) = result {
            let res = json_response_ok(&state, &user);
            Ok((state, res))
        } else {
            let res = json_response_bad_message(&state, "invalid token".into());
            Ok((state, res))
        }
    }
    .boxed()
}

#[derive(Debug, Serialize)]
struct OkBool {
    ok: bool,
}

/// PUT /api/v1/confirm/:token
pub fn confirm_user_email(state: State) -> Pin<Box<HandlerFuture>> {
    let token = {
        let path = TokenPath::borrow_from(&state).clone();
        path.token.to_owned()
    };
    let repo = Repo::borrow_from(&state).clone();

    async move {
        let result = repo
            .run(move |conn| verify_email_with_token(&conn, token.as_str()))
            .await;

        match result {
            Ok(b) => {
                let res = json_response_ok(&state, &OkBool { ok: b });
                Ok((state, res))
            }
            Err(_) => {
                let res =
                    json_response_bad_message(&state, "Email belonging to token not found.".into());
                Ok((state, res))
            }
        }
    }
    .boxed()
}

/// Handles `PUT /user/:user_id/resend` route
pub fn regenerate_token_and_send(state: State) -> Pin<Box<HandlerFuture>> {
    let user = UserPath::borrow_from(&state);
    // get current user id
    let token = AuthorizationToken::<Claims>::borrow_from(&state);
    let current_user_id = token.0.claims.user_id();

    if user.id != current_user_id {
        let res =
            json_response_bad_message(&state, "current user does not match requested user.".into());

        return future::ok((state, res)).boxed();
    }

    let repo = Repo::borrow_from(&state).clone();

    async move {
        let result = repo
            .run(move |conn| regenerate_email_token_and_send(&conn, current_user_id))
            .await;

        match result {
            Ok(b) => {
                let res = json_response_ok(&state, &OkBool { ok: b });
                Ok((state, res))
            }
            Err(_) => {
                let res =
                    json_response_bad_message(&state, "Email belonging to token not found.".into());
                Ok((state, res))
            }
        }
    }
    .boxed()
}
