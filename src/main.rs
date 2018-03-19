#[macro_use] extern crate error_chain;
#[macro_use] extern crate serde_derive;
#[macro_use(crate_version, crate_authors)] extern crate clap;
extern crate toml;
extern crate serde;
extern crate shellexpand;
extern crate exec;
extern crate cursive;
extern crate either;
extern crate itertools;

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use clap::{App, Arg, AppSettings};

use config::ConfigItem;
use execution::Execution;

mod config;
mod ui;
mod execution;

fn main() {
    let matches = App::new("sshc")
        .version(crate_version!())
        .author(crate_authors!())
        .about("A simple SSH connections manager.")
        .setting(AppSettings::ColoredHelp)
        .arg(
            Arg::from_usage("-c, --config=[FILE] 'Path to the configuration file'")
                .default_value("~/.config/sshc/config.toml")
        )
        .args_from_usage(
            "-p, --profile=[PROFILE] 'Run the specified profile immediately'
             -d, --dry-run 'Just print the command'"
        )
        .get_matches();

    let config_path = matches.value_of("config").unwrap();
    let config_path: Cow<Path> = str_to_path(config_path);

    let config = match config::load(&config_path) {
        Ok(config) => config,
        Err(e) => {
            println!("Failed to load configuration from {}: {}", config_path.display(), e);
            std::process::exit(1)
        }
    };

    let dry_run = matches.is_present("dry-run");

    if let Some(profile) = matches.value_of("profile") {
        let parts: Vec<_> = profile.split(".").collect();

        let mut group = config.root;
        let mut definition = None;
        for (i, part) in parts.iter().cloned().enumerate() {
            match group.definitions.remove(part) {
                Some(ConfigItem::Definition(d)) =>
                    if i == parts.len() - 1 {
                        definition = Some(d);
                    } else {
                        break;
                    },
                Some(ConfigItem::Subgroup(g)) =>
                    if i < parts.len() - 1 {
                        group = g;
                    } else {
                        break;
                    },
                _ => {}
            }
        }

        if let Some(definition) = definition {
            let mut e = Execution::from(definition);
            if dry_run {
                println!("{}", e.command_line());
            } else {
                e.run();
            }

        } else {
            eprintln!("Invalid profile name: {}", profile);
            std::process::exit(1);
        }

    } else {
        ui::run(config, dry_run);
    }
}

#[inline]
pub fn str_to_path(s: &str) -> Cow<Path> {
    match shellexpand::tilde(s) {
        Cow::Borrowed(s) => Path::new(s).into(),
        Cow::Owned(s) => PathBuf::from(s).into(),
    }
}

#[inline]
pub fn string_to_path(s: &String) -> Cow<Path> { str_to_path(s) }
