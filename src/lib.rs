#[allow(unused)]
#[macro_use]
use std::process;

use hyper::Server;
use hyper::rt::Future;
use log::{error, info};
use mongodb::{Client}

use crate::config::Config;
use crate::mongodb::{client_from_str}
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
        let conn_str: &str = self.config.server.mongodb[..]
        let client = match client_from_str(conn_str) {
            Ok(client) => client,
            Err(e)  => {
                error!("Failed to connect to mongodb: {}", e);
                process::exit(0x0100);
            }
        }

        gotham::start(self.config.server.address, http::router(client))
    }
}
