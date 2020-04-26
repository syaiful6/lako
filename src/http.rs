use gotham::pipeline::new_pipeline;
use gotham::pipeline::set::{finalize_pipeline_set, new_pipeline_set};
use gotham::router::{builder::*, Router};
use gotham_middleware_diesel::DieselMiddleware;
use gotham::state::State;
use gotham_middleware_jwt::JWTMiddleware;

use crate::auth::{Claims, get_jwt_secret_key};
use crate::db::{Repo};
use crate::routes::auth::{register_user_handler, login_user_handler, get_user};

const HELLO_WORLD: &str = "Hello World!";

fn say_hello(state: State) -> (State, &'static str) {
    (state, HELLO_WORLD)
}

pub fn router(repo: Repo) -> Router {
    // Add the diesel middleware to a new pipeline
    let pipelines = new_pipeline_set();
    let (pipelines, default) = pipelines.add(
        new_pipeline()
            .add(DieselMiddleware::new(repo))
            .build(),
    );
    let (pipelines, authenticated) = pipelines.add(
        new_pipeline()
            .add(JWTMiddleware::<Claims>::new(get_jwt_secret_key()))
            .build(),
    );
    // finalize this
    let pipeline_set = finalize_pipeline_set(pipelines);
    let default_chain = (default, ());
    let auth_chain = (authenticated, default_chain);
    
    build_router(default_chain, pipeline_set, |route| {
        route.get("/").to(say_hello);
        route.scope("/api/v1", |route| {
            // public route
            route.post("/register").to(register_user_handler);
            route.post("/login").to(login_user_handler);
            // route that need to protected
            route.with_pipeline_chain(auth_chain, |route| {
                route.get("/me").to(get_user);
            });
        });
    })
}