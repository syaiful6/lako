use gotham::pipeline::new_pipeline;
use gotham::pipeline::set::{finalize_pipeline_set, new_pipeline_set};
use gotham::router::{builder::*, Router};
use gotham::state::State;
use gotham_middleware_diesel::DieselMiddleware;
use gotham_middleware_jwt::JwtMiddleware;

use crate::auth::{get_jwt_secret_key, Claims};
use crate::db::Repo;
use crate::routes::auth::{
    confirm_user_email, get_user, login_user_handler, regenerate_token_and_send,
    register_user_handler, user_update_detail_handler,
};
use crate::routes::clients::{
    create_client_handler, delete_client_handler, list_client_handler, update_client_handler,
};
use crate::routes::companies::{
    create_company_handler, delete_company_handler, list_company_handler, update_company_handler,
};
use crate::routes::invoices::create_invoice_handler;
use crate::routes::paths::{PaginationExtractor, ResourceIDPath, TokenPath};

const HELLO_WORLD: &str = "Hello World!";

fn say_hello(state: State) -> (State, &'static str) {
    (state, HELLO_WORLD)
}

pub fn router(repo: Repo) -> Router {
    // Add the diesel middleware to a new pipeline
    let pipelines = new_pipeline_set();
    let (pipelines, default) =
        pipelines.add(new_pipeline().add(DieselMiddleware::new(repo)).build());
    let (pipelines, authenticated) = pipelines.add(
        new_pipeline()
            .add(JwtMiddleware::<Claims>::new(get_jwt_secret_key()))
            .build(),
    );
    // finalize this
    let pipeline_set = finalize_pipeline_set(pipelines);
    let default_chain = (default, ());
    let auth_chain = (authenticated, default_chain);

    build_router(default_chain, pipeline_set, |route| {
        route.get("/").to(say_hello);
        // api routes
        route.scope("/api/v1", |route| {
            // public route
            route.post("/register").to(register_user_handler);
            route.post("/login").to(login_user_handler);
            route
                .put("/confirm/:token")
                .with_path_extractor::<TokenPath>()
                .to(confirm_user_email);

            // route that need to protected
            route.with_pipeline_chain(auth_chain, |route| {
                route.get("/me").to(get_user);
                route.patch("/me").to(user_update_detail_handler);

                // scope user
                route.scope("/users", |route| {
                    route
                        .put("/:id/resend")
                        .with_path_extractor::<ResourceIDPath>()
                        .to(regenerate_token_and_send);
                });

                route.scope("/clients", |route| {
                    route.post("/").to(create_client_handler);
                    route
                        .get("/")
                        .with_query_string_extractor::<PaginationExtractor>()
                        .to(list_client_handler);

                    route
                        .patch("/:id")
                        .with_path_extractor::<ResourceIDPath>()
                        .to(update_client_handler);

                    route
                        .delete("/:id")
                        .with_path_extractor::<ResourceIDPath>()
                        .to(delete_client_handler);
                });

                route.scope("/companies", |route| {
                    route.post("/").to(create_company_handler);
                    route
                        .get("/")
                        .with_query_string_extractor::<PaginationExtractor>()
                        .to(list_company_handler);

                    route
                        .patch("/:id")
                        .with_path_extractor::<ResourceIDPath>()
                        .to(update_company_handler);

                    route
                        .delete("/:id")
                        .with_path_extractor::<ResourceIDPath>()
                        .to(delete_company_handler);
                });

                route.scope("/invoices", |route| {
                    route.post("/").to(create_invoice_handler);
                })
            });
        });
    })
}
