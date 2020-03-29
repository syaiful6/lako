use gotham::pipeline::{new_pipeline, single::single_pipeline};
use gotham::router::{builder::*, Router};
use gotham_middleware_diesel::DieselMiddleware;
use gotham::state::State;

use crate::db::{Repo};
use crate::routes::auth::{register_user_handler};

const HELLO_WORLD: &str = "Hello World!";

fn say_hello(state: State) -> (State, &'static str) {
    (state, HELLO_WORLD)
}

pub fn router(repo: Repo) -> Router {
    // Add the diesel middleware to a new pipeline
    let (chain, pipeline) =
        single_pipeline(new_pipeline().add(DieselMiddleware::new(repo)).build());
    
    build_router(chain, pipeline, |route| {
        route.get("/").to(say_hello);
        route.scope("/private", |route| {
            route.post("/register").to(register_user_handler);
        });
    })
}