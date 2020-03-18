use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::builder::*;
use gotham::router::Router;
use mongodb::{Client}
use crate::mongodb::{MongoClientMiddleware};

/// The handler which is invoked for all requests to "/".
///
/// This handler expects that `ExampleMiddleware` has already been executed by Gotham before
/// it is invoked. As a result of that middleware being run our handler trusts that it must
/// have placed data into state that we can perform operations on.
pub fn middleware_reliant_handler(mut state: State) -> (State, Response<Body>) {
    // Finally we create a basic Response to complete our handling of the Request.
    let res = create_empty_response(&state, StatusCode::OK);
    (state, res)
}

/// Create a `Router`
fn router(client: Client) -> Router {
    // Within the Gotham web framework Middleware is added to and referenced from a Pipeline.
    //
    // A pipeline can consist of multiple Middleware types and guarantees to call them all in the
    // ordering which is established by successive calls to the `add` method.
    //
    // A pipeline is considered complete once the build method is called and can no longer
    // be modified.
    //
    // The Gotham web framework supports multiple Pipelines and even Pipelines containing Pipelines.
    // However, as shown here, many applications will get sufficent power and flexibility
    // from a `single_pipeline` which we've provided specific API assitance for.
    let (chain, pipelines) = single_pipeline(new_pipeline()
        .add(MongoClientMiddleware::new(client))
        .build());

    // Notice we've switched from build_simple_router which has been present in all our examples up
    // until this point. Under the hood build_simple_router has simply been creating an empty
    // set of Pipelines on your behalf.
    //
    // Now that we're creating and populating our own Pipelines we'll switch to using
    // build_router directly.
    //
    // Tip: Use build_simple_router for as long as you can. Switching to build_router is simple once
    // do need to introduce Pipelines and Middleware.
    build_router(chain, pipelines, |route| {
        route.get("/").to(middleware_reliant_handler);
    })
}