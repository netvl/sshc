sshc: A simple SSH connection manager
=====================================

`sshc` is a very simple ncurses-based SSH connection manager.

How to build
------------

This program uses [Cargo] for build management. Just run `cargo build` and the program
will be available as `./target/sshc`. Of course, you will need ncurses library installed.

  [cargo]: http://crates.io

How to use
----------

Copy `sshc.toml` file to your `~/.ssh` directory and run `sshc` binary.
It will present you a menu based on `sshc.toml` contents. Use up/down (or k/j) keys
to select desired host and press enter to invoke `ssh` binary with corresponding arguments.
Press `q` to exit from the program.

Configuration file
------------------

Configuration file is a [TOML] document which consists of items of the following format:
```toml
# File format:
[hosts.<item-name>]
host = "<host address or IP>"
port = <SSH port number>
user = "<user name>"
key = "<path to key file>"
```

All fields but `host` are optional, and safe defaults are used:
* `port` - default is 22;
* `user` - default is the current user;
* `key`  - default is disabled.

Item name is used for informative purposes only; the menu is sorted by it, so you can use
it to reorder your items for your convenience.

  [toml]: https://github.com/toml-lang/toml

Licence
-------

This program is licensed under MIT license.

---
Copyright (C) Vladimir Matveev, 2014

