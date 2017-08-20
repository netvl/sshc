use std::char;
use std::num::Zero;
use std::os;

use ncurses::*;

use data;
use exec;

struct State {
    hosts: data::Hosts,
    selected: uint
}

impl State {
    #[inline]
    fn new(hosts: data::Hosts) -> State {
        State {
            hosts: hosts,
            selected: 0
        }
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.selected < self.hosts.hosts.len()-1 {
            self.selected += 1;
        }
    }

    fn find_selected(&self) -> &String {
        for (n, k) in self.hosts.hosts.keys().enumerate() {
            if n == self.selected { return k; }
        }
        unreachable!()
    }
}

fn num_len<N: Div<N, N>+Zero+PartialEq+FromPrimitive>(mut n: N) -> uint {
    let mut r = 0;
    while { n = n / FromPrimitive::from_u8(10).unwrap(); r += 1; n != Zero::zero() } {}
    r
}

fn compute_widths(hosts: &data::Hosts) -> (uint, uint, uint, uint) {
    if hosts.hosts.is_empty() { return (0, 0, 0, 0) }

    let number_width = num_len(hosts.hosts.len());
    let name_width = hosts.hosts.keys().map(|k| k.len()).max().unwrap();
    let addr_width = hosts.hosts.values()
        .map(|v| v.user.len() + v.host.len() + num_len(v.port) as uint + 2)
        .max().unwrap();
    let key_width = hosts.hosts.values()
        .flat_map(|v| v.key.as_ref().map(|k| k[]).into_iter())
        .map(|s| s.len())
        .max();
    let max_width = number_width + 2 + name_width + 3 + addr_width + 
                    key_width.map(|w| 3 + w).unwrap_or(0);
    (number_width, name_width, addr_width, max_width)
}

fn render(state: &mut State) {
    let (number_width, name_width, addr_width, max_width) = compute_widths(&state.hosts);
    
    for (n, (name, h)) in state.hosts.hosts.iter().enumerate() {
        let (c_x, c_y) = (5, 3+n as i32);

        let addr = format!("{}@{}:{}", h.user, h.host, h.port);
        let mut s = format!("{0:1$u}. {2:3$s} - {4:5$s}", 
                            n+1, number_width, name[], name_width, addr, addr_width);
        if let Some(ref key) = h.key {
            s.push_str(" - ");
            s.push_str(key[]);
        }

        mvprintw(c_y, c_x, s[]);

        mv(c_y, c_x);
        if n == state.selected {
            chgat(max_width as i32, A_REVERSE(), 0);
        } else {
            chgat(max_width as i32, A_NORMAL(), 0);
        }
    }

    mvprintw(3 + state.hosts.hosts.len() as i32 + 3, 5, 
             "q to exit, up/down/k/j to move selection, enter to confirm");
}

fn execute(state: &mut State) -> ! {
    endwin();

    let args = state.hosts.hosts[*state.find_selected()].to_cmd_line();
    fn wrap_quotes_if_needed(s: &String) -> String {
        if s[].find(char::is_whitespace).is_some() {
            format!("'{}'", s)
        } else {
            s.clone()
        }
    }
    println!("Executing ssh {}...", 
             args.iter().map(wrap_quotes_if_needed).collect::<Vec<String>>().connect(" "));

    exec::exec("ssh", args);
    fail!("Cannot execute ssh: {}", os::last_os_error());
}

fn react(state: &mut State) -> bool {
    macro_rules! one_of(
        ($e:expr ~ $($p:expr),+) => (
            $($e == $p as i32 ||)+ false
        )
    )   
    let ch = getch();
    match ch {
        c if one_of!(c ~ KEY_UP,   b'k') => state.move_up(),
        c if one_of!(c ~ KEY_DOWN, b'j') => state.move_down(),
        c if c == b'\n' as i32 => execute(state),
        c if c == b'q' as i32 => return false,
        _ => {}
    }
    true
}

fn init_ui() {
    initscr();
    cbreak();
    keypad(stdscr, true);
    noecho();
}

pub fn start(hosts: data::Hosts) {
    init_ui();

    let mut state = State::new(hosts);
    loop {
        render(&mut state);
        if !react(&mut state) {
            break;
        }
    }

    endwin();
}

#[cfg(test)]
mod tests {
    use super::{num_len, compute_widths};

    use data;

    macro_rules! smap(
        ($($k:expr -> $v:expr),*) => ({
            let mut hm = ::std::collections::TreeMap::new();
            $(hm.insert($k.to_string(), $v);)*
            hm
        })
    )

    #[test]
    fn test_num_len() {
        assert_eq!(1, num_len(0u));
        assert_eq!(1, num_len(1i));
        assert_eq!(1, num_len(7u16));
        assert_eq!(2, num_len(82i32));
        assert_eq!(16, num_len(1234567812345678u64));
    }

    #[test]
    fn test_compute_widths() {
        let h = data::Hosts {
            hosts: smap![
                "first-name" -> data::Host::new("abcd", 22, "host-1", Some("key.pem")),
                "second-name" -> data::Host::new("de", 1234, "longer-host", None)
            ]
        };
        assert_eq!((1, 11, 19, 46), compute_widths(&h));
    }
}
