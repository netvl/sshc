use std::c_str::CString;
use std::os;
use std::ptr;

use libc::{c_char, execvp};

pub fn exec(name: &str, args: Vec<String>) {
    let args: Vec<CString> = 
        Some(name.to_c_str()).into_iter()
        .chain(args.into_iter().map(|s| s.to_c_str()))
        .collect();
    let mut args: Vec<*const c_char> = 
        args.iter().map(|s| s.as_ptr() as *const c_char)
        .chain(Some(ptr::null()).into_iter())
        .collect();
    let argv: *mut *const c_char = args.as_mut_ptr();

    let name: CString = name.to_c_str();
    let file: *const c_char = name.as_ptr();

    unsafe { execvp(file, argv); }
    fail!("Error executing {}: {}", name, os::last_os_error());
}
