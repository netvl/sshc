#![feature(phase)]

extern crate libc;
extern crate serialize;

extern crate toml;

#[phase(plugin)] extern crate docopt_macros;
extern crate docopt;

use docopt::FlagParser;

mod config;
mod data;
mod user;

docopt!(Args deriving Show, "
Usage: sshc [--help] [--version] [-c CONFIG]

Options:
    -c CONFIG, --config CONFIG  Path to configuration file. [default: ~/.ssh/sshc.toml]
    -h, --help                  Show this message.
    -v, --version               Show version info.
")

fn main() {
    let args: Args = FlagParser::parse_conf(docopt::Config {
        options_first: true,
        help: true,
        version: Some("sshc 0.0.1".to_string())
    }).unwrap_or_else(|e| e.exit());

    let config_path = Path::new(args.flag_config);

    let config = match config::Config::load(&config_path) {
        Ok(config) => config,
        Err(e) => {
            println!("Cannot load configuration: {}", e);
            return;
        }
    };

    let hosts = data::Hosts::from_config(config);
    println!("{}", hosts);
}
