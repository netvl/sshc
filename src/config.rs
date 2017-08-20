use std::fmt;
use std::fs::File;
use std::path::Path;
use std::collections::BTreeMap;
use std::io::{self, Read};

use toml;
use toml::Value;
use toml::value::{Table, Array};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SingleJump {
    pub host: String,
    pub port: u16,
    pub user: Option<String>,
    pub key: Option<String>,
    pub tunnel: Option<Tunnel>,
    pub verbose: bool,
    pub agent_passthrough: bool,
    pub no_command: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct Tunnel {
    pub local_port: Option<u16>,
    pub local_host: Option<String>,
    pub remote_port: Option<u16>,
    pub remote_host: Option<String>
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigDefinition {
    pub chain: Vec<SingleJump>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigGroup {
    pub definitions: BTreeMap<String, ConfigItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigItem {
    Definition(ConfigDefinition),
    Subgroup(ConfigGroup),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub root: ConfigGroup,
}

error_chain! {
    foreign_links {
        Io(io::Error);
        Toml(toml::de::Error);
    }
}

pub fn load(path: &Path) -> Result<Config> {
    let mut f = File::open(path)?;

    let mut data = String::new();
    f.read_to_string(&mut data)?;

    load_from_string(&data)
}

fn load_from_string(s: &str) -> Result<Config> {
    let table = match s.parse::<Value>()? {
        Value::Table(table) => table,
        _ => unreachable!(),  // cannot happen
    };
    let root = read_config_group("".into(), table)?;

    Ok(Config { root, })
}

fn read_config_group(path: String, table: Table) -> Result<ConfigGroup> {
    let mut definitions = BTreeMap::new();

    fn mkpath(first: &str, second: &str) -> String {
        if first.is_empty() { second.into() }
        else { first.to_owned() + "." + second }
    }

    for (k, v) in table {
        let item = match v {
            Value::Table(table) =>
                ConfigItem::Subgroup(read_config_group(mkpath(&path, &k), table)?),
            Value::Array(array) =>
                ConfigItem::Definition(read_config_definition(mkpath(&path, &k), array)?),
            other =>
                return Err(format!(
                    "unexpected config item {} in {}, expected table or array, got {}",
                    k, path, other.type_str()
                ).into()),
        };
        definitions.insert(k, item);
    }

    Ok(ConfigGroup { definitions, })
}

fn read_config_definition(path: String, array: Array) -> Result<ConfigDefinition> {
    let mut chain = Vec::new();
    for (idx, item) in array.into_iter().enumerate() {
        let item = match item {
            Value::Table(table) => SingleJumpContext::new(&path, idx).read_from_table(table)?,
            Value::String(string) => SingleJumpContext::new(&path, idx).read_from_string(string)?,
            other => return Err(format!(
                "unexpected jump configuration in {}, expected table or string, got {}",
                path, other.type_str()
            ).into()),
        };
        chain.push(item);
    }
    Ok(ConfigDefinition { chain, })
}

struct SingleJumpContext<'a> {
    path: &'a str,
    idx: usize,
}

impl<'a> SingleJumpContext<'a> {
    fn new(path: &'a str, idx: usize) -> SingleJumpContext<'a> {
        SingleJumpContext { path: path.into(), idx, }
    }

    fn err<T, S: AsRef<str>>(&self, msg: S) -> Result<T> {
        Err(format!("jump {} of {}: {}", self.idx + 1, self.path, msg.as_ref()).into())
    }

    fn read_from_string(&self, s: String) -> Result<SingleJump> {
        let table = Some(("host".to_owned(), Value::String(s))).into_iter().collect();
        self.read_from_table(table)
    }

    fn read_from_table(&self, mut table: Table) -> Result<SingleJump> {
        let host = match table.remove("host") {
            Some(Value::String(host)) => host,
            Some(other) => return self.err(format!("host is invalid: expected string, got {}", other.type_str())),
            None => return self.err("host is missing"),
        };

        let HostInfo { host, port, user, key, } = self.parse_host(&host)?;

        let host = host.into();

        let port = match table.remove("port") {
            Some(Value::Integer(i)) if i >= u16::min_value() as i64 && i <= u16::max_value() as i64 => i as u16,
            None => port.unwrap_or(22),
            Some(Value::Integer(i)) =>
                return self.err(format!("port is invalid: expected number from 0 to 65536, got {}", i)),
            Some(other) =>
                return self.err(format!("port is invalid: expected number from 0 to 65536, got {}", other.type_str()))
        };

        let user = match table.remove("user") {
            Some(Value::String(u)) => Some(u),
            None => user.map(Into::into),
            Some(other) =>
                return self.err(format!("user is invalid: expected string, got {}", other.type_str()))
        };

        let key = match table.remove("key") {
            Some(Value::String(k)) => Some(k),
            None => key.map(Into::into),
            Some(other) =>
                return self.err(format!("key is invalid: expected string, got {}", other.type_str()))
        };

        let tunnel = match table.remove("tunnel") {
            Some(Value::Table(table)) => Some(self.tunnel_from_table(table)?),
            Some(Value::String(string)) => Some(self.tunnel_from_string(string)?),
            Some(Value::Integer(integer)) => Some(self.tunnel_from_integer(integer)?),
            None => None,
            Some(other) =>
                return self.err(format!("tunnel is invalid: expected string or table, got {}", other.type_str()))
        };

        let verbose = match table.remove("verbose") {
            Some(Value::Boolean(v)) => v,
            None => false,
            Some(other) => return self.err(format!("verbose is invalid: expected boolean, got {}", other.type_str())),
        };

        let agent_passthrough = match table.remove("agent_passthrough") {
            Some(Value::Boolean(a)) => a,
            None => false,
            Some(other) => return self.err(format!("agent_passthrough is invalid: expected boolean, got {}", other.type_str())),
        };

        let no_command = match table.remove("no_command") {
            Some(Value::Boolean(n)) => n,
            None => false,
            Some(other) => return self.err(format!("no_command is invalid: expected boolean, got {}", other.type_str())),
        };

        Ok(SingleJump { host, port, user, key, tunnel, verbose, agent_passthrough, no_command, })
    }

    fn tunnel_from_string(&self, s: String) -> Result<Tunnel> {
        const ERR: &str = "tunnel is invalid: expected table or '[local_host]:[local_port]|[remote_host]:[remote_port]'";

        if s.chars().filter(|c| *c == '|').count() != 1 {
            return self.err(ERR);
        }

        let mut parts = s.split("|");
        let (local, remote) = (parts.next().unwrap(), parts.next().unwrap());

        let parse_host_port = |s: &str| {
            // Does not work with IPv6 yet

            if s.chars().filter(|c| *c == ':').count() != 1 {
                return self.err(ERR);
            }
            let mut parts = s.split(":");

            let (host, port) = (parts.next().unwrap(), parts.next().unwrap());

            let port = if port.trim().is_empty() {
               None
            } else {
                match port.parse() {
                    Ok(p) => Some(p),
                    Err(e) => return self.err(format!("tunnel is invalid: port is invalid: {}", e)),
                }
            };

            let host = if host.trim().is_empty() {
                None
            } else {
                Some(host.into())
            };

            Ok((host, port))
        };

        let (local_host, local_port) = parse_host_port(local)?;
        let (remote_host, remote_port) = parse_host_port(remote)?;

        Ok(Tunnel { local_host, local_port, remote_host, remote_port, })
    }

    fn tunnel_from_integer(&self, i: i64) -> Result<Tunnel> {
        const MIN: i64 = ::std::u16::MIN as i64;
        const MAX: i64 = ::std::u16::MAX as i64;
        match i {
            MIN...MAX => Ok(Tunnel {
                local_host: None,
                local_port: Some(i as u16),
                remote_host: None,
                remote_port: None,
            }),
            _ => self.err(format!("tunnel is invalid: port number is out of range: {}", i)),
        }
    }

    fn tunnel_from_table(&self, t: Table) -> Result<Tunnel> {
        match Value::Table(t).try_into() {
            Ok(t) => Ok(t),
            Err(e) => self.err(format!("tunnel is invalid: {}", e)),
        }
    }

    fn parse_host<'h>(&self, host: &'h str) -> Result<HostInfo<'h>> {
        if host.chars().filter(|c| *c == ':').count() > 2 {
            return self.err("host is invalid: expected [user@]host[:[port][:key]]");
        }
        
        let mut parts = host.split(':');
        let (host, port, user, key) = match (parts.next().unwrap(), parts.next(), parts.next()) {
            (host, None, None) => {  // just host
                let (host, user) = parse_user_and_host(host);
                (host, None, user, None)
            },
            (host, Some(port), key) => {  // host and port
                let (host, user) = parse_user_and_host(host);
                let port = if port.trim().is_empty() { None } else {
                    match port.parse::<u16>() {
                        Ok(port) => Some(port),
                        Err(e) => return self.err(format!("port is invalid: {}", e)),
                    }
                };
                (host, port, user, key)
            },
            (_, None, Some(_)) => unreachable!(),
        };

        Ok(HostInfo { host, port, user, key, })
    }

}

struct HostInfo<'a> {
    host: &'a str,
    port: Option<u16>,
    user: Option<&'a str>,
    key: Option<&'a str>,
}

fn parse_user_and_host(s: &str) -> (&str, Option<&str>) {
    let mut parts = s.splitn(2, "@");
    match (parts.next().unwrap(), parts.next()) {
        (user, Some(host)) => (host, Some(user)),
        (host, None) => (host, None),
    }
}

#[cfg(test)]
mod tests {
    const TEST_DATA: &str = r#"
be_3 = ["user@be-3.example.com:2244:~/.ssh/be.pem"]
be_4 = [{ host = "be-3.example.com", port = 1234, key = "/bla/bla.pem" }]
be_5 = ["user@be-5.example.com::~/.ssh/be.pem"]

[[be_1]]
host = "be-1.example.com"
port = 2222
user = "user"
key = "~/.ssh/be.pem"
verbose = true

[[be_2]]
host = "be-2.example.com"

[my.a.b]
googolplex = [
  { host = "some.server", port = 2222, user = "user" },
  { host = "serverplex:1234:/a/b/c.pem" },
  { host = "googolplex:1234", key = "~/whatever.pem" }
]

[my]
whatever1 = [
  { host = "serverplex", tunnel = 12345 },
  { host = "transplex" },  # taken from above
]

whatever2full = [
  { host = "serverplex", tunnel = ":1221|:4433" },
  { host = "transplex", tunnel = ":4443|:443" }
]

whatever2alternative = [{ host = "serverplex", tunnel = ":1221|transplex:443" }]

whatever4 = [
  { host = "host1" },
  { host = "host2" },
  { host = "host3", tunnel = 12345 },  # falls "down", incl. agent_passthrough = true
  { host = "host4" },
  { host = "host5" }
]

whatever2 = [
    { host = "serverplex", tunnel = ":1221|:4443" },
    { host = "transplex", tunnel = ":4443|:443" }
]

whatever2simpler = [
    { host = "serverplex", tunnel = 12345 },
    { host = "transplex" }
]

whatever3 = [
    { host = "host1", tunnel = 12345 },
    { host = "host2" },
    { host = "host3" },
    { host = "host4" }
]
    "#;

    use super::*;

    #[test]
    fn test_parsing() {
        let config = load_from_string(TEST_DATA).unwrap();

        let mut root = config.root.definitions;

        // be_3 = ["user@be-3.example.com:2244:~/.ssh/be.pem"]
        assert_eq!(
            root.remove("be_3").unwrap(),
            ConfigItem::Definition(ConfigDefinition {
                chain: vec![
                    SingleJump {
                        host: "be-3.example.com".into(),
                        port: 2244,
                        user: Some("user".into()),
                        key: Some("~/.ssh/be.pem".into()),
                        tunnel: None,
                        verbose: false,
                        agent_passthrough: false,
                        no_command: false,
                    }
                ]
            })
        );

        // be_4 = [{ host = "be-3.example.com", port = 1234, key = "/bla/bla.pem" }]
        assert_eq!(
            root.remove("be_4").unwrap(),
            ConfigItem::Definition(ConfigDefinition {
                chain: vec![
                    SingleJump {
                        host: "be-3.example.com".into(),
                        port: 1234,
                        user: None,
                        key: Some("/bla/bla.pem".into()),
                        tunnel: None,
                        verbose: false,
                        agent_passthrough: false,
                        no_command: false,
                    }
                ]
            })
        );

        let mut my = match root.remove("my").unwrap() {
            ConfigItem::Subgroup(my) => my.definitions,
            other => panic!("Invalid subgroup my: {:?}", other),
        };

        // whatever2 = [
        //     { host = "serverplex", tunnel = ":1221|:4443" },
        //     { host = "transplex", tunnel = ":4443|:443" }
        // ]
        assert_eq!(
            my.remove("whatever2").unwrap(),
            ConfigItem::Definition(ConfigDefinition {
                chain: vec![
                    SingleJump {
                        host: "serverplex".into(),
                        port: 22,
                        user: None,
                        key: None,
                        tunnel: Some(Tunnel {
                            local_host: None,
                            local_port: Some(1221),
                            remote_host: None,
                            remote_port: Some(4443),
                        }),
                        verbose: false,
                        agent_passthrough: false,
                        no_command: false,
                    },
                    SingleJump {
                        host: "transplex".into(),
                        port: 22,
                        user: None,
                        key: None,
                        tunnel: Some(Tunnel {
                            local_host: None,
                            local_port: Some(4443),
                            remote_host: None,
                            remote_port: Some(443),
                        }),
                        verbose: false,
                        agent_passthrough: false,
                        no_command: false,
                    },
                ]
            })
        );

    }
}
