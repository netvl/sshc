use std::os;

pub fn expand_tilde(p: String) -> String {
    match p[].find('~') {
        Some(idx) => match os::homedir() {
            Some(path) => format!("{}{}{}", p[..idx], path.display(), p[idx+1..]),
            None => p
        },
        None => p
    }
}
