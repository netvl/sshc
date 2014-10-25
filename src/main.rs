#![feature(macro_rules, phase, if_let, globs, slicing_syntax)]

extern crate libc;
extern crate serialize;
#[phase(plugin)] extern crate docopt_macros;
extern crate docopt;
extern crate ncurses;
extern crate toml;

use docopt::FlagParser;

mod config;
mod data;
mod exec;
mod ui;
mod user;
mod util;

const VERSION: &'static str = "0.0.1";

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
        version: Some(format!("sshc {}", VERSION))
    }).unwrap_or_else(|e| e.exit());

    let config_path = Path::new(util::expand_tilde(args.flag_config));

    let config = match config::Config::load(&config_path) {
        Ok(config) => config,
        Err(e) => {
            println!("Cannot load configuration: {}", e);
            return;
        }
    };

    let hosts = data::Hosts::from_config(config);

    ui::start(hosts);   
}
