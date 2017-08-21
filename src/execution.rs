use std::fmt;

use itertools::Itertools;
use exec;

use config::{ConfigDefinition, Tunnel, State};

pub struct Execution {
    definition: ConfigDefinition,
    command_parts: Vec<Vec<String>>,
}

impl From<ConfigDefinition> for Execution {
    fn from(definition: ConfigDefinition) -> Self {
        let definition = normalize_definition(definition);
        Execution { definition, command_parts: Vec::new(), }
    }
}

impl Execution {
    pub fn prepare(&mut self) {
        let mut parts = Vec::new();

        for jump in &self.definition.chain {
            let mut cmd = vec!["ssh".into()];

            if jump.verbose {
                cmd.push("-v".into());
            }

            if let State::Enabled(_) = jump.agent_passthrough {
                cmd.push("-A".into());
            }

            if let Some(port) = jump.port {
                cmd.push("-p".into());
                cmd.push(port.to_string());
            }

            if let Some(ref key) = jump.key {
                cmd.push("-i".into());
                cmd.push(key.clone());
            }

            if let State::Enabled(ref tunnel) = jump.tunnel {
                cmd.push("-L".into());
                cmd.push(SshArg(tunnel).to_string());
            }

            if let Some(ref user) = jump.user {
                cmd.push(format!("{}@{}", user, jump.host));
            } else {
                cmd.push(jump.host.clone());
            }

            if let State::Enabled(_) = jump.no_command {
                cmd.push("-N".into());
            }

            parts.push(cmd);
        }

        self.command_parts = parts;
    }

    pub fn command_line(&mut self) -> String {
        if self.command_parts.is_empty() {
            self.prepare();
        }

        self.command_parts.iter()
            .map(|part| part.iter().join(" "))
            .join(" -t \\\n  ")
    }

    pub fn run(mut self) {
        if self.command_parts.is_empty() {
            self.prepare();
        }

        let args: Vec<_> = self.command_parts.into_iter()
            .intersperse(vec!["-t".into()])
            .flatten()
            .collect();

        println!("{}", args.join(" "));

        let command = args[0].clone();
        let error = exec::execvp(command, args);

        eprintln!("Failed to run SSH: {}", error);
        ::std::process::exit(1);
    }
}

/// Expands the absent pieces in the configuration.
///
/// In particular, does the following things:
/// * Expands tunnelspecs, e.g. 12345 -> ":12345|localhost:12345"
/// * Propagates tunnelspecs down the chains
/// * Adds flags responsible for -A and -N where appropriate
fn normalize_definition(mut definition: ConfigDefinition) -> ConfigDefinition {
    let mut last_tunnel: Option<Tunnel> = None;
    let chain_len = definition.chain.len();
    for (i, jump) in definition.chain.iter_mut().enumerate() {
        // Fix the tunnelspec if it is present:
        //   1. Propagate local or remote port to its missing counterpart
        //   2. Add the default "localhost" value if the remote host is absent
        if let State::Enabled(ref mut tunnel) = jump.tunnel {
            if tunnel.remote_host.is_none() {
                tunnel.remote_host = Some("localhost".into());
            }

            if tunnel.local_port.is_none() {
                tunnel.local_port = tunnel.remote_port;
            }

            if tunnel.remote_port.is_none() {
                tunnel.remote_port = tunnel.local_port;
            }
        }

        // Inject the previous tunnel if it is absent in this jump, unless it is explicitly disabled
        if let State::Unset = jump.tunnel {
            if let Some(ref last_tunnel) = last_tunnel {
                jump.tunnel = State::Enabled(last_tunnel.clone());
            }
        }

        // If the tunnel is configured
        if let State::Enabled(ref tunnel) = jump.tunnel {
            // Add the `-A` argument if this is not the last jump and if it is not disabled explicitly
            if i < chain_len - 1 {
                if let State::Unset = jump.agent_passthrough {
                    jump.agent_passthrough = State::Enabled(());
                }
            }

            // Add the `-N` argument if this is the last jump and if it is not disabled explicitly
            if i == chain_len - 1 {
                if let State::Unset = jump.no_command {
                    jump.no_command = State::Enabled(());
                }
            }

            // Save the tunnel configuration for further use
            last_tunnel = Some(tunnel.clone());

        } else {
            // Remove the last tunnel if it is not configured/disabled in this jump
            last_tunnel = None;
        }
    }

    definition
}

struct SshArg<'a>(&'a Tunnel);

impl<'a> fmt::Display for SshArg<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Tunnel { ref local_host, local_port, ref remote_host, remote_port } = *self.0;

        if let Some(host) = local_host.as_ref() {
            write!(f, "{}:", host)?;
        }

        if let Some(port) = local_port {
            write!(f, "{}:", port)?;
        }

        if let Some(host) = remote_host.as_ref() {
            f.write_str(host)?;
        }

        f.write_str(":")?;

        if let Some(port) = remote_port.as_ref() {
            write!(f, "{}", port)?;
        }

        Ok(())
    }
}
