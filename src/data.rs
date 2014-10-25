use std::collections::TreeMap;

use config;
use user;
use util;

#[deriving(Show)]
pub struct Hosts {
    pub hosts: TreeMap<String, Host>
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

macro_rules! extend(
    ($target:expr <- $($arg:expr),+) => ({
        $($target.push($arg));+
    })
)

impl Host {
    #[inline]
    pub fn new(host: &str, port: u16, user: &str, key: Option<&str>) -> Host {
        Host {
            host: host.to_string(),
            port: port, 
            user: user.to_string(),
            key: key.map(|s| s.to_string())
        }
    }
    
    pub fn from_config(config: config::Host, name: &str) -> Host { 
        Host {
            host: config.host,
            port: config.port.unwrap_or(22),
            user: config.user.unwrap_or_else(|| name.to_string()),
            key: config.key
        }
    }

    pub fn to_cmd_line(&self) -> Vec<String> {
        // ssh [-i key] -p port user@host
        let mut args = Vec::new();

        if let Some(ref key) = self.key {
            extend!(args <- "-i".to_string(), util::expand_tilde(key.clone()));
        }

        extend!(args <- "-p".to_string(), self.port.to_string());
        extend!(args <- format!("{}@{}", self.user, self.host));

        args
    }
}

#[cfg(test)]
mod tests {
    use super::Host;

    macro_rules! svec(
        ($($e:expr),*) => (vec![$($e.to_string()),*])
    )

    #[test]
    fn test_to_cmd_line() {
        let m = Host {
            host: "localhost".to_string(),
            port: 1234,
            user: "user".to_string(),
            key: None
        };
        assert_eq!(svec!["-p", "1234", "user@localhost"], m.to_cmd_line());

        let m = Host {
            host: "localhost".to_string(),
            port: 1234,
            user: "user".to_string(),
            key: Some("~/.ssh/key.pem".to_string())
        };
        assert_eq!(svec!["-i", "~/.ssh/key.pem", "-p", "1234", "user@localhost"], 
                   m.to_cmd_line());
    }
}
