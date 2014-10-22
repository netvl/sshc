use std::collections::HashMap;
use std::io::{mod, File};
use std::fmt;

use serialize::Decodable;

use toml;

#[deriving(Show, Clone, Decodable, PartialEq)]
pub struct Config {
    pub hosts: HashMap<String, Host>
}

#[deriving(Show, Clone, Decodable, PartialEq)]
pub struct Host {
    pub host: String,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub key: Option<String>
}

pub enum ConfigError {
    ParserError(Vec<toml::ParserError>),
    DecodeError(toml::DecodeError),
    IoError(io::IoError)
}

impl fmt::Show for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError(ref errs) => {
                try!(f.write(b"parsing error: "));
                for e in errs.iter() {
                    try!(write!(f, "{}; ", e));
                }
                Ok(())
            }
            DecodeError(ref err) => write!(f, "decode error: {}", err),
            IoError(ref err) => write!(f, "i/o error: {}", err)
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Config, ConfigError> {
        let mut file = try!(File::open(path).map_err(IoError));
        let data = try!(file.read_to_string().map_err(IoError));
        
        let mut parser = toml::Parser::new(data.as_slice());
        let table = match parser.parse() {
            Some(table) => table,
            None => return Err(ParserError(parser.errors))
        };

        let mut decoder = toml::Decoder::new(toml::Table(table));
        Decodable::decode(&mut decoder).map_err(DecodeError)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{File, TempDir};

    use super::{Config, Host};

    #[test]
    fn test_load() {
        static CONFIG: &'static str = r#"
            [hosts.be_1]
            host = "be-1.example.com"
            port = 2222
            user = "user"
            key = "~/.ssh/be.pem"

            [hosts.be_2]
            host = "be-2.example.com"
            # port = 22
            # user = current user
            # key = None
        "#;

        let dir = TempDir::new("sshc-config").unwrap();
        let fname = dir.path().join("config.toml");

        let mut file = File::create(&fname).unwrap();
        file.write(CONFIG.as_bytes()).unwrap();
        drop(file);

        assert_eq!(
            Config::load(&fname).unwrap(),
            Config {
                hosts: vec![
                    ("be_1".to_string(), Host {
                        host: "be-1.example.com".to_string(),
                        port: Some(2222),
                        user: Some("user".to_string()),
                        key: Some("~/.ssh/be.pem".to_string())
                    }),
                    ("be_2".to_string(), Host {
                        host: "be-2.example.com".to_string(),
                        port: None,
                        user: None,
                        key: None
                    })
                ].into_iter().collect()
            }
        )
    }
}
