#![allow(unused)]
//#![feature(plugin)]
//#![plugin(docopt_macros)]
//
//extern crate libc;
//extern crate rustc_serialize;
//extern crate docopt;
//extern crate ncurses;
//extern crate toml;
//
//mod config;
//mod data;
//mod exec;
//mod ui;
//mod user;
//mod util;
//
//const VERSION: Option<&'static str> = env_option!("CARGO_PACKAGE_VERSION");
//
//docopt!(Args deriving Show, "
//Usage: sshc [--help] [--version] [-c CONFIG]
//
//Options:
//    -c CONFIG, --config CONFIG  Path to configuration file. [default: ~/.ssh/sshc.toml]
//    -h, --help                  Show this message.
//    -v, --version               Show version info.
//")
//
//fn main() {
//    let args: Args = FlagParser::parse_conf(docopt::Config {
//        options_first: true,
//        help: true,
//        version: Some(format!("sshc {}", VERSION))
//    }).unwrap_or_else(|e| e.exit());
//
//    let config_path = Path::new(util::expand_tilde(args.flag_config));
//
//    let config = match config::Config::load(&config_path) {
//        Ok(config) => config,
//        Err(e) => {
//            println!("Cannot load configuration: {}", e);
//            return;
//        }
//    };
//
//    let hosts = data::Hosts::from_config(config);
//
//    ui::start(hosts);
//}
extern crate toml;
extern crate serde;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate serde_derive;

mod config;

fn main() {

}
