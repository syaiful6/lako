#[macro_use]
extern crate diesel;

#[allow(unused)]
#[macro_use]
extern crate gotham_derive;

#[macro_use]
extern crate validator_derive;

use log::{error, info};
use std::process;

use crate::config::Config;
use crate::db::create_repo;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod email;
pub mod http;
pub mod models;
pub mod routes;
pub mod schema;
pub mod sql_types;
pub(crate) mod sqlx;

pub fn bootstrap() {
    let cfg = match config::load_configuration() {
        Ok(cfg) => cfg,
        Err(e) => {
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
        let addr = self.config.server.address.to_string();

        gotham::start(addr, http::router(create_repo(&self.config)))
    }
}
