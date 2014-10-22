use std::collections::HashMap;

use config;
use user;

#[deriving(Show)]
pub struct Hosts {
    pub hosts: HashMap<String, Host>
}

#[deriving(Show)]
pub struct Host {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub key: Option<String>
}

impl Hosts {
    pub fn from_config(config: config::Config) -> Hosts {
        // obtain current user name
        let user = user::current_user_data().unwrap();
        let name = user.info().name;

        let hosts = config.hosts.into_iter()
            .map(|(k, v)| (k, Host::from_config(v, name)))
            .collect();
        Hosts { hosts: hosts }
    }
}

impl Host {
    pub fn from_config(config: config::Host, name: &str) -> Host { 
        Host {
            host: config.host,
            port: config.port.unwrap_or(22),
            user: config.user.unwrap_or_else(|| name.to_string()),
            key: config.key
        }
    }
}
