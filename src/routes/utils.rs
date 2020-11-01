use std::str::from_utf8;

use gotham::handler::{HandlerError, MapHandlerError, MapHandlerErrorFuture};
use gotham::helpers::http::response::create_response;
use gotham::hyper::{body, Body, Response, StatusCode};
use gotham::state::{FromState, State};
use serde_derive::Serialize;

pub async fn extract_json<T>(state: &mut State) -> Result<T, HandlerError>
where
    T: serde::de::DeserializeOwned,
{
    let body = body::to_bytes(Body::take_from(state))
        .map_err_with_status(StatusCode::BAD_REQUEST)
        .await?;
    let b = body.to_vec();
    from_utf8(&b)
        .map_err_with_status(StatusCode::BAD_REQUEST)
        .and_then(|s| serde_json::from_str::<T>(s).map_err_with_status(StatusCode::BAD_REQUEST))
}

pub fn json_response<T: serde::Serialize>(
    state: &State,
    t: &T,
    status_code: StatusCode,
) -> Response<Body> {
    let body = serde_json::to_string(t).unwrap();

    create_response(state, status_code, mime::APPLICATION_JSON, body)
}

pub fn json_response_ok<T: serde::Serialize>(state: &State, t: &T) -> Response<Body> {
    json_response(state, t, StatusCode::OK)
}

#[derive(Debug, Serialize)]
struct ErrMessage {
    message: String,
}

pub fn json_response_bad_message(state: &State, msg: String) -> Response<Body> {
    json_response(state, &ErrMessage { message: msg }, StatusCode::BAD_REQUEST)
}
