#[allow(unused)]
#[macro_use]
use std::process;

use hyper::Server;
use hyper::rt::Future;
use log::{error, info};

use crate::config::Config;

mod config;
mod http;


pub fn bootstrap() {
    let cfg = match config::load_configuration() {
        Ok(cfg) => cfg,
        Err(e)  => {
            error!("Failed to load configuration: {}", e);
            process::exit(0x0100);
        }
    };
    
    // start the app!
    let app = Lako::new(cfg);
    app.run();
}

pub struct Lako {
    config: Config,
}

impl Lako {
    pub fn new(cfg: Config) -> Lako {
        Lako { config: cfg }
    }

    pub fn run(&self) {
        info!("Starting Lako");

        let addr = self.config.server.address.parse().unwrap();

        let server = Server::bind(&addr)
            .serve(http::router_service)
            .map_err(|e| eprintln!("server error: {}", e));

        info!("Listening on http://{}", addr);

        hyper::rt::run(server);
    }
}
