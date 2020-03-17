use std::collections::HashMap;
use std::fmt;

use clap::{App, Arg};
use log::error;
use serde_derive::{Deserialize, Serialize};

// Server default address
const DEFAULT_SERVER_ADDRESS: &str = "0.0.0.0:9999";

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    #[serde(default = "HashMap::new")]
    pub log: HashMap<String, Log>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Server {
    pub address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Log {
    pub name: Option<String>,
    pub datastores: Vec<String>,
    pub commit_window: String,
}

impl Config {
    // create new config
    pub fn new(server: Server) -> Config {
        Config {
            server: server,
            log: HashMap::new(),
        }
    }

    pub fn get_log(&self, logname: &String) -> Option<&Log> {
        self.log.get(&logname[..])
    }

    pub fn commit_window_to_seconds(commit_window: &String) -> Option<u64> {
        let last_character = &commit_window[commit_window.len() - 1..commit_window.len()];
        match last_character {
            "s" => {
                let integer_value = &commit_window[0..commit_window.len() - 1].parse::<u64>();
                let seconds = match integer_value {
                    Ok(val) => Some(*val),
                    Err(_) => {
                        error!("Interval cannot be parsed");
                        None
                    }
                };
                seconds
            }
            "m" => {
                let integer_value = &commit_window[0..commit_window.len() - 1].parse::<u64>();
                let seconds = match integer_value {
                    Ok(val) => Some(*val * 60),
                    Err(_) => {
                        error!("Interval cannot be parsed");
                        None
                    }
                };
                seconds
            }
            _ => None,
        }
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
    let matches = App::new("Deploy")
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

    let server = Server {
        address,
    };

    let configuration = Config::new(server);

    // always return Ok for now
    Ok(configuration)
}