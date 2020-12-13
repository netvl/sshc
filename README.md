# sshc: A simple SSH connection manager

**Unmaintained: consider using SSH's own configuration system, like host aliases, instead.**

---

`sshc` is a very simple SSH connection manager with a textual dialog-based UI. Basically, it provides a config file format and a UI which turns this config into an `ssh` command execution. The coniguration file allows defining chains of connections, as well as port forwarding.

## How to build

This program uses Cargo for build management, and it is published to [crates.io][cargo]. Run `cargo install sshc` to get the program installed into your `~/.cargo/bin` directory. sshc uses [`termion`] as the underlying TUI engine, therefore it does not depend on any third-party libraries like ncurses.

  [cargo]: http://crates.io
  [termion]: https://github.com/ticki/termion/
  
## How to use

Create a `config.toml` file in the `~/.config/sshc` directory and run `sshc`.

It will present you a menu based on `sshc.toml` contents. Use up/down and Enter to navigate the tree of run profiles. Use Esc to go up the tree and to exit when at the root level. Alternatively, you can specify the profile name directly in the command line:

```
$ sshc -p my.server
```

In this approach you can also pass the `-d` argument to do a dry run (sshc will only print the command which will be executed).

## Configuration file

Configuration file is a [TOML] document which consists of items of the following format:
```toml
[[group.subgroup.item]]
host = "<host definition>"
port = <port number>
user = "<user name>"
key = "<path to the public key file>"
tunnel = <tunnel specification (see below)>
verbose = true/false                       
agent_passthrough = true/false             
no_command = true/false                    
```

All fields except `host` are optional. Also, the `host` field may be of the following format:

```
# Full
user@host:12345:/home/user/keys/for-host.pem

# No port and user
host::/home/user/keys/for-host.pem

# No key
user@host:2222
```

In other words, you can define username, port and public key as a part of the host definition. If some parameter is defined both as a part of the host and in the configuration table, then values defined in the table take priority.

The `verbose` option maps directly to the `-v` flag of the `ssh` command, `agent_passthrough` maps to `-A` and `no_command` maps to `-N`. That said, how sshc handles the latter two flags differs depending on whether the port forwarding (`tunnel`) is configured.

The above example of the configuration section is intentionally written in the long format; usually you can define it in a much shorter way, for example:

```toml
[group.subgroup]
item1 = ["user@server"]
item2 = [{ host = "server", port = 2222, verbose = true }]
```

Note that item definitions are always arrays. This is also intentional, because items actually define *chains* of `ssh` invocations.

  [toml]: https://github.com/toml-lang/toml

### Chains

Consider this profile:

```toml
home_server = [
    "public-vps.cc",
    "home-server.vpn",
]
```

sshc transforms it into the following SSH command:

```
ssh public-vps.cc -t ssh home-server.vpn
```

In other words, if you specify several records in a profile, they will be joined into a single SSH command, connected with `-t`. This would allow interactive connection to the last host in the chain, as well as password prompts on all of the hosts in the middle.

### Tunnels

Consider this profile:

```toml
transmission_ui = [
    { host = "public-vpn.cc", tunnel = 9091 },
    { host = "home-server.vpn" }
]
```

(Note that you have to define both items as tables. This is a limitation of the TOML format.)

sshc transforms it into the following SSH command:

```
ssh -A -L 9091:localhost:9091 public-vpn.cc -t ssh -L 9091:localhost:9091 -N
```

The `tunnel` definition is transformed into a `-L` port forwarding argument for the SSH command. Also, it is propagated "down" the chain, and `-A` and `-N` arguments are added. The rules for this propagation are as follows. For each single jump definition inside a chain, if a `tunnel` is configured in the current definition:

1. the `tunnel` configuration is copied into the next jump definition, unless it is already defined there or is disabled via `tunnel = false`;
2. if the current jump is not the last one in the chain, then the `agent_passthrough` option is enabled in this definition, unless it is explicitly disabled via `agent_passthrough = false`;
3. if the current jump is the last one in the chain, then the `no_command` option is enabled in this definition, unless it is explicitly disabled via `no_command = false`.

Note that by default `agent_passthrough` is not enabled for the last jump, because port-forwarding chains are usually non-interactive and therefore do not need an agent. You can enabled it explicitly if you need it.

Also, each `tunnel` configuration is expanded according to the following rules:

1. if the remote host is not specified, it is assumed to be `"localhost"`;
2. if either the local port or remote port is defined but its counterpart is not, then the latter is set to the value of the former, e.g. if the local port is `2222` and the remote one is not set, it is assumed to be also `2222`.

These rules lead to a natural expansion of simple definitions like `tunnel = 9091` into definitions commonly used for port forwarding: `tunnel = ":9091|localhost:9091"`.

License
-------

This program is licensed under the MIT license.

---
Copyright (C) Vladimir Matveev, 2014-2017

