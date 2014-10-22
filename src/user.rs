use std::c_str::CString;
use std::mem;
use std::ptr;

use libc::{c_char, c_int, uid_t, gid_t, size_t};

#[repr(C)]
struct Passwd {
    name:   *const c_char,
    passwd: *const c_char,
    uid:    uid_t,
    gid:    gid_t,
    gecos:  *const c_char,
    dir:    *const c_char,
    shell:  *const c_char
}

pub struct PasswdData {
    passwd: Passwd,
    _data: Vec<c_char>
}

impl PasswdData {
    fn get_str<'a>(&'a self, p: *const c_char) -> &'a str {
        unsafe {
            mem::transmute(CString::new(p, false).as_str().unwrap())
        }
    }

    pub fn info(&self) -> PasswdInfo {
        PasswdInfo {
            name: self.get_str(self.passwd.name),
            passwd: self.get_str(self.passwd.passwd),
            uid: self.passwd.uid as uint,
            gid: self.passwd.gid as uint,
            gecos: self.get_str(self.passwd.gecos),
            dir: self.get_str(self.passwd.dir),
            shell: self.get_str(self.passwd.shell)
        }
    }
}

pub struct PasswdInfo<'a> {
    pub name: &'a str,
    pub passwd: &'a str,
    pub uid: uint,
    pub gid: uint,
    pub gecos: &'a str,
    pub dir: &'a str,
    pub shell: &'a str
}

extern {
    fn geteuid() -> uid_t;
    fn getpwuid_r(uid: uid_t, pwd: *mut Passwd,
                  buf: *mut c_char, buflen: size_t,
                  result: *mut *const Passwd) -> c_int;
}

pub fn current_user_data() -> Option<PasswdData> {
    const SIZE_MAX: uint = 16384;  // TODO: load proper value through sysconf() 

    let mut pwd = unsafe { mem::uninitialized() };
    let mut data = Vec::with_capacity(SIZE_MAX);
    unsafe { data.set_len(SIZE_MAX); }

    let mut result: *const Passwd = ptr::null();
    let _ = unsafe { 
        getpwuid_r(geteuid(), &mut pwd, 
                   data.as_mut_ptr(), data.len() as size_t, 
                   &mut result)
    };
    if result.is_null() {
        None  // TODO: check for error?
    } else {
        Some(PasswdData {
            passwd: pwd,
            _data: data
        })
    }
}

