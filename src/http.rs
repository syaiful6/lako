use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::{Request, Response, Body, Method};
use hyper_router::{Route, RouterBuilder, RouterService};

fn request_handler(_: Request<Body>) -> Response<Body> {
    let body = "Hello World";
    Response::builder()
        .header(CONTENT_LENGTH, body.len() as u64)
        .header(CONTENT_TYPE, "text/plain")
        .body(Body::from(body))
        .expect("Failed to construct the response")
}

pub fn router_service() -> Result<RouterService, std::io::Error> {
    let router = RouterBuilder::new()
        .add(Route::get("/hello").using(request_handler))
        .add(Route::from(Method::PATCH, "/asd").using(request_handler))
        .build();
    
    Ok(RouterService::new(router))
}