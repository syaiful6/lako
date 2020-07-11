use std::str::from_utf8;

use gotham::helpers::http::response::create_response;
use gotham::state::{FromState, State};
use futures::{Future, Stream};
use gotham::handler::{HandlerError, IntoHandlerError};
use hyper::{Body, Response, StatusCode};
use serde_derive::Serialize;

pub fn bad_request<E>(e: E) -> HandlerError
where
    E: std::error::Error + Send + 'static,
{
    e.into_handler_error().with_status(StatusCode::BAD_REQUEST)
}


pub fn extract_json<T>(state: &mut State) -> impl Future<Item = T, Error = HandlerError>
where
    T: serde::de::DeserializeOwned,
{
    Body::take_from(state)
        .concat2()
        .map_err(bad_request)
        .and_then(|body| {
            let b = body.to_vec();
            from_utf8(&b)
                .map_err(bad_request)
                .and_then(|s| serde_json::from_str::<T>(s).map_err(bad_request))
        })
}


pub fn json_response<T: serde::Serialize>(state: &State, t: &T, status_code: StatusCode) -> Response<Body> {
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
