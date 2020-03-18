#[macro_use]
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::io;
use std::process;

use futures::future::{self, Future};
use log::{error, trace};
use gotham_derive::StateData;
use gotham::handler::HandlerFuture;
use gotham::middleware::{Middleware, NewMiddleware};
use gotham::state::{request_id, State};
use mongodb::{Client, ClientOptions}


pub fn client_from_str(s: &str) -> Option<Client> {
    let mut client_options = ClientOptions::parse(s)?;
    client_options.app_name = Some("Lako".to_string());
    // Get a handle to the deployment.
    Client::with_options(client_options)
}

#[derive(StateData, Clone)]
pub struct MongoClient {
    pub client: Client,
}

pub struct MongoClientMiddleware {
    client: AssertUnwindSafe<Client>,
}

impl MongoClientMiddleware {
    pub fn new(client: Client) -> Self {
        MongoClientMiddleware {
            client: AssertUnwindSafe(client),
        }
    }
}

impl Clone for MongoClientMiddleware {
    fn clone(&self) -> self {
        match catch_unwind(|| self.client.clone()) {
            Ok(client) => MongoClientMiddleware {
                client: AssertUnwindSafe(client),
            }
        }
    }
}

impl NewMiddleware for MongoClientMiddleware {
    type Instance = MongoClientMiddleware;

    fn new_middleware(&self) -> io::Result<Self::Instance> {
        match catch_unwind(|| self.client.clone()) {
            Ok(repo) => Ok(MongoClientMiddleware {
                client: AssertUnwindSafe(client),
            }),
            Err(_) => {
                error!(
                    "PANIC: r2d2::Pool::clone caused a panic, unable to rescue with a HTTP error"
                );
                process::abort()
            }
        }
    }
}

impl Middleware for MongoClientMiddleware {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture> + 'static,
        Self: Sized,
    {
        trace!("[{}] pre chain", request_id(&state));
        state.put(self.client.clone());

        let f = chain(state).and_then(move |(state, response)| {
            {
                trace!("[{}] post chain", request_id(&state));
            }
            future::ok((state, response))
        });
        Box::new(f)
    }
}