use std::env;
use std::fmt;

use clap::{App, Arg};
use serde_derive::{Deserialize, Serialize};

// Server default address
const DEFAULT_SERVER_ADDRESS: &str = "0.0.0.0:8000";

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub server: Server,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Server {
    pub address: String,
    pub db_url: String,
}

impl Config {
    // create new config
    pub fn new(server: Server) -> Config {
        Config { server: server }
    }
}

#[derive(Debug)]
pub struct ConfigurationError {
    details: String,
}

impl ConfigurationError {
    pub fn new(msg: &str) -> ConfigurationError {
        ConfigurationError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

// Loads the configuration file from command
pub fn load_configuration() -> Result<Config, ConfigurationError> {
    let matches = App::new("Lako")
        .version("0.0.1")
        .about("Project management tools")
        .arg(
            Arg::with_name("address")
                .takes_value(true)
                .default_value(DEFAULT_SERVER_ADDRESS)
                .short("a")
                .long("address")
                .help("Server binding address")
                .required(true),
        )
        .get_matches();

    let address = matches.value_of("address").unwrap().to_string();

    let db_url: String = match env::var("DATABASE_URL") {
        Ok(val) => val,
        Err(err) => {
            return Err(ConfigurationError::new(&format!(
                "No Database URL environment variable `DATABASE_URL` set. {}",
                err
            )))
        }
    };

    let server = Server { address, db_url };

    let configuration = Config::new(server);

    Ok(configuration)
}
