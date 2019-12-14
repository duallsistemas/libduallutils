use crypto::digest::Digest;
use crypto::md5::Md5;
use crypto::sha1::Sha1;
use libc::{c_char, c_int, size_t};
use opener;
use std::ffi::CString;
use std::fs::File;
use std::io::ErrorKind::NotFound;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::ptr;

mod utils;

#[cfg(target_os = "windows")]
const BUFFER_SIZE: usize = 4096; /* 4k */
#[cfg(not(target_os = "windows"))]
const BUFFER_SIZE: usize = 16384; /* 16k */

/// Retrieves library version as C-like string.
#[no_mangle]
pub unsafe extern "C" fn du_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), '\0').as_ptr() as *const c_char
}

/// Frees the C-lik string `cstr` from the memory.
///
/// # Arguments
///
/// * `[in] cstr` - C-like string to be freed.
#[no_mangle]
pub unsafe extern "C" fn du_dispose(cstr: *mut c_char) {
    if !cstr.is_null() {
        CString::from_raw(cstr);
    }
}

/// Generates a MD5 from a given string.
///
/// # Arguments
///
/// * `[in] cstr` - Given C-like string.
/// * `[in,out] md5` - Generated MD5.
/// * `[in] size` - Size of the `cstr` string.
///
/// # Returns
///
/// * `0` - Success.
/// * `-1` - Invalid argument.
#[no_mangle]
pub unsafe extern "C" fn du_md5(cstr: *const c_char, md5: *mut c_char, size: size_t) -> c_int {
    if cstr.is_null() || md5.is_null() || size <= 0 {
        return -1;
    }
    let mut hasher = Md5::new();
    hasher.input_str(from_c_str!(cstr).unwrap());
    let hash = to_c_str!(hasher.result_str()).unwrap();
    copy_c_str!(hash, md5, size);
    0
}

/// Generates a MD5 from a given file.
///
/// # Arguments
///
/// * `[in] cstr` - Filename as C-like string.
/// * `[in,out] md5` - Generated MD5.
/// * `[in] size` - Size of the `cstr` string.
///
/// # Returns
///
/// * `0` - Success.
/// * `-1` - Invalid argument.
/// * `-2` - File not found.
/// * `-3` - Unknown error.
#[no_mangle]
pub unsafe extern "C" fn du_md5_file(
    filename: *const c_char,
    md5: *mut c_char,
    size: size_t,
) -> c_int {
    if filename.is_null() || md5.is_null() || size <= 0 {
        return -1;
    }
    match File::open(from_c_str!(filename).unwrap()) {
        Ok(mut file) => {
            let mut hasher = Md5::new();
            let mut buf = [0u8; BUFFER_SIZE];
            loop {
                let n = match file.read(&mut buf) {
                    Ok(n) => n,
                    Err(_) => return -3,
                };
                hasher.input(&buf[..n]);
                if n == 0 || n < BUFFER_SIZE {
                    break;
                }
            }
            let hash = to_c_str!(hasher.result_str()).unwrap();
            copy_c_str!(hash, md5, size);
        }
        Err(e) => {
            if e.kind() == NotFound {
                return -2;
            }
            return -3;
        }
    }
    0
}

/// Generates a SHA-1 from a given string.
///
/// # Arguments
///
/// * `[in] cstr` - Given C-like string.
/// * `[in,out] sha1` - Generated SHA-1.
/// * `[in] size` - Size of the `cstr` string.
///
/// # Returns
///
/// * `0` - Success.
/// * `-1` - Invalid argument.
#[no_mangle]
pub unsafe extern "C" fn du_sha1(cstr: *const c_char, sha1: *mut c_char, size: size_t) -> c_int {
    if cstr.is_null() || sha1.is_null() || size <= 0 {
        return -1;
    }
    let mut hasher = Sha1::new();
    hasher.input_str(from_c_str!(cstr).unwrap());
    let hash = to_c_str!(hasher.result_str()).unwrap();
    copy_c_str!(hash, sha1, size);
    0
}

/// Generates a SHA-1 from a given file.
///
/// # Arguments
///
/// * `[in] cstr` - Filename as C-like string.
/// * `[in,out] sha1` - Generated SHA-1.
/// * `[in] size` - Size of the `cstr` string.
///
/// # Returns
///
/// * `0` - Success.
/// * `-1` - Invalid argument.
/// * `-2` - File not found.
/// * `-3` - Unknown error.
#[no_mangle]
pub unsafe extern "C" fn du_sha1_file(
    filename: *const c_char,
    sha1: *mut c_char,
    size: size_t,
) -> c_int {
    if filename.is_null() || sha1.is_null() || size <= 0 {
        return -1;
    }
    match File::open(from_c_str!(filename).unwrap()) {
        Ok(mut file) => {
            let mut hasher = Sha1::new();
            let mut buf = [0u8; BUFFER_SIZE];
            loop {
                let n = match file.read(&mut buf) {
                    Ok(n) => n,
                    Err(_) => return -3,
                };
                hasher.input(&buf[..n]);
                if n == 0 || n < BUFFER_SIZE {
                    break;
                }
            }
            let hash = to_c_str!(hasher.result_str()).unwrap();
            copy_c_str!(hash, sha1, size);
        }
        Err(e) => {
            if e.kind() == NotFound {
                return -2;
            }
            return -3;
        }
    }
    0
}

/// Executes the command as a child process.
///
/// # Arguments
///
/// * `[in] program` - Program path as C-like string.
/// * `[in] workdir` - Working directory as C-like string.
/// * `[in] args` - Arguments to pass to the program as array of C-like string.
/// * `[in] envs` - Environment variables to pass to the program as array of C-like string.
/// * `[in] waiting` - Waiting for the program to exit completely.
/// * `[in,out] exitcode` - Exit code of the process.
///
/// # Returns
///
/// * `0` - Success.
/// * `-1` - Invalid argument.
/// * `-2` - Program not found.
/// * `-3` - Unknown error.
#[no_mangle]
pub unsafe extern "C" fn du_spawn(
    program: *const c_char,
    workdir: *const c_char,
    args: *const *const c_char,
    envs: *const *const c_char,
    waiting: bool,
    exitcode: *mut c_int,
) -> c_int {
    if program.is_null() {
        return -1;
    }
    let mut cmd = Command::new(from_c_str!(program).unwrap());
    if !workdir.is_null() {
        cmd.current_dir(from_c_str!(workdir).unwrap());
    }
    if cfg!(test) {
        cmd.stdout(Stdio::null());
    }
    if !args.is_null() {
        cmd.args(&from_c_array!(args));
    }
    if !envs.is_null() {
        for i in 0.. {
            let env = *(envs.offset(i));
            if env == ptr::null() {
                break;
            }
            let pair: Vec<&str> = from_c_str!(env).unwrap().splitn(2, "=").collect();
            if pair.len() == 2 {
                cmd.env(pair.first().unwrap(), pair.last().unwrap());
            }
        }
    }
    match cmd.spawn() {
        Ok(mut child) => {
            if waiting {
                match child.try_wait() {
                    Ok(Some(status)) => *exitcode = status.code().unwrap(),
                    Ok(None) => *exitcode = child.wait().unwrap().code().unwrap(),
                    Err(_) => return -3,
                }
            }
        }
        Err(e) => {
            if e.kind() == NotFound {
                return -2;
            }
            return -3;
        }
    }
    0
}

/// Executes the command as a child process waiting for it to finish and collecting all of its output.
///
/// # Arguments
///
/// * `[in] program` - Program path as C-like string.
/// * `[in] workdir` - Working directory as C-like string.
/// * `[in] args` - Arguments to pass to the program as array of C-like string.
/// * `[in] envs` - Environment variables to pass to the program as array of C-like string.
/// * `[in,out] output` - C-like string containing the `stdout` content if it exists.
/// * `[in,out] error` - C-like string containing the `stderr` content if it exists.
/// * `[in,out] exitcode` - Exit code of the process.
///
/// # Returns
///
/// * `0` - Success.
/// * `-1` - Invalid argument.
/// * `-2` - Program not found.
/// * `-3` - Unknown error.
#[no_mangle]
pub unsafe extern "C" fn du_execute(
    program: *const c_char,
    workdir: *const c_char,
    args: *const *const c_char,
    envs: *const *const c_char,
    output: *mut *mut c_char,
    error: *mut *mut c_char,
    exitcode: *mut c_int,
) -> c_int {
    if program.is_null() {
        return -1;
    }
    let mut cmd = Command::new(from_c_str!(program).unwrap());
    if !workdir.is_null() {
        cmd.current_dir(from_c_str!(workdir).unwrap());
    }
    if !args.is_null() {
        cmd.args(&from_c_array!(args));
    }
    if !envs.is_null() {
        for i in 0.. {
            let env = *(envs.offset(i));
            if env == ptr::null() {
                break;
            }
            let pair: Vec<&str> = from_c_str!(env).unwrap().splitn(2, "=").collect();
            if pair.len() == 2 {
                cmd.env(pair.first().unwrap(), pair.last().unwrap());
            }
        }
    }
    match cmd.output() {
        Ok(child) => {
            *output = to_c_str!(String::from_utf8(child.stdout).unwrap())
                .unwrap()
                .into_raw();
            *error = to_c_str!(String::from_utf8(child.stderr).unwrap())
                .unwrap()
                .into_raw();
            *exitcode = child.status.code().unwrap();
        }
        Err(e) => {
            if e.kind() == NotFound {
                return -2;
            }
            return -3;
        }
    }
    0
}

/// Opens a file or link with the system default program.
///
/// # Arguments
///
/// * `[in] path` - File or link as C-like string.
///
/// # Returns
///
/// * `0` - Success.
/// * `-1` - Invalid argument.
/// * `-2` - File not found.
/// * `-3` - Unknown error.
#[no_mangle]
pub unsafe extern "C" fn du_open(path: *const c_char) -> c_int {
    if path.is_null() {
        return -1;
    }
    let p = from_c_str!(path).unwrap();
    if !Path::new(p).exists() {
        return -2;
    }
    match opener::open(p) {
        Ok(_) => 0,
        Err(_) => -3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version() {
        unsafe {
            assert_eq!(
                from_c_str!(du_version()).unwrap(),
                env!("CARGO_PKG_VERSION")
            );
        }
    }

    #[test]
    fn md5() {
        unsafe {
            let hash: [c_char; 33] = [0; 33];
            assert_eq!(du_md5(ptr::null(), hash.as_ptr() as *mut c_char, 33), -1);
            assert_eq!(
                du_md5(to_c_str!("abc123").unwrap().as_ptr(), ptr::null_mut(), 33),
                -1
            );
            assert_eq!(
                du_md5(
                    to_c_str!("abc123").unwrap().as_ptr(),
                    hash.as_ptr() as *mut c_char,
                    0
                ),
                -1
            );
            assert_eq!(
                du_md5(
                    to_c_str!("abc123").unwrap().as_ptr(),
                    hash.as_ptr() as *mut c_char,
                    hash.len()
                ),
                0
            );
            assert_eq!(
                from_c_str!(hash.as_ptr()).unwrap(),
                "e99a18c428cb38d5f260853678922e03"
            );
        }
    }

    #[test]
    fn md5_file() {
        unsafe {
            let hash: [c_char; 33] = [0; 33];
            assert_eq!(
                du_md5_file(ptr::null(), hash.as_ptr() as *mut c_char, 33),
                -1
            );
            assert_eq!(
                du_md5_file(
                    to_c_str!("abc123.txt").unwrap().as_ptr(),
                    ptr::null_mut(),
                    33
                ),
                -1
            );
            assert_eq!(
                du_md5_file(
                    to_c_str!("abc123.txt").unwrap().as_ptr(),
                    hash.as_ptr() as *mut c_char,
                    0
                ),
                -1
            );
            assert_eq!(
                du_md5_file(
                    to_c_str!("blah blah").unwrap().as_ptr(),
                    hash.as_ptr() as *mut c_char,
                    33
                ),
                -2
            );
            assert_eq!(
                du_md5_file(
                    to_c_str!("LICENSE").unwrap().as_ptr(),
                    hash.as_ptr() as *mut c_char,
                    hash.len()
                ),
                0
            );
            assert_eq!(
                from_c_str!(hash.as_ptr()).unwrap(),
                "d88e9e08385d2a17052dac348bde4bc1"
            );
        }
    }

    #[test]
    fn sha1() {
        unsafe {
            let hash: [c_char; 41] = [0; 41];
            assert_eq!(du_sha1(ptr::null(), hash.as_ptr() as *mut c_char, 41), -1);
            assert_eq!(
                du_sha1(to_c_str!("abc123").unwrap().as_ptr(), ptr::null_mut(), 41),
                -1
            );
            assert_eq!(
                du_sha1(
                    to_c_str!("abc123").unwrap().as_ptr(),
                    hash.as_ptr() as *mut c_char,
                    0
                ),
                -1
            );
            assert_eq!(
                du_sha1(
                    to_c_str!("abc123").unwrap().as_ptr(),
                    hash.as_ptr() as *mut c_char,
                    hash.len()
                ),
                0
            );
            assert_eq!(
                from_c_str!(hash.as_ptr()).unwrap(),
                "6367c48dd193d56ea7b0baad25b19455e529f5ee"
            );
        }
    }

    #[test]
    fn sha1_file() {
        unsafe {
            let hash: [c_char; 41] = [0; 41];
            assert_eq!(
                du_sha1_file(ptr::null(), hash.as_ptr() as *mut c_char, 41),
                -1
            );
            assert_eq!(
                du_sha1_file(
                    to_c_str!("abc123.txt").unwrap().as_ptr(),
                    ptr::null_mut(),
                    41
                ),
                -1
            );
            assert_eq!(
                du_sha1_file(
                    to_c_str!("abc123.txt").unwrap().as_ptr(),
                    hash.as_ptr() as *mut c_char,
                    0
                ),
                -1
            );
            assert_eq!(
                du_sha1_file(
                    to_c_str!("blah blah").unwrap().as_ptr(),
                    hash.as_ptr() as *mut c_char,
                    41
                ),
                -2
            );
            assert_eq!(
                du_sha1_file(
                    to_c_str!("LICENSE").unwrap().as_ptr(),
                    hash.as_ptr() as *mut c_char,
                    hash.len()
                ),
                0
            );
            assert_eq!(
                from_c_str!(hash.as_ptr()).unwrap(),
                "6d842099530d126dea37db858a755e444f4de3f7"
            );
        }
    }

    #[test]
    fn spawn() {
        unsafe {
            let mut code: c_int = 0;
            assert_eq!(
                du_spawn(
                    ptr::null(),
                    ptr::null(),
                    ptr::null(),
                    ptr::null(),
                    true,
                    &mut code
                ),
                -1
            );
            assert_eq!(
                du_spawn(
                    to_c_str!("blah blah").unwrap().as_ptr(),
                    ptr::null(),
                    ptr::null(),
                    ptr::null(),
                    true,
                    &mut code
                ),
                -2
            );
            assert_eq!(
                du_spawn(
                    to_c_str!("echo").unwrap().as_ptr(),
                    ptr::null(),
                    ptr::null(),
                    ptr::null(),
                    true,
                    &mut code
                ),
                0
            );
            assert_eq!(code, 0);
        }
    }

    #[test]
    fn execute() {
        unsafe {
            let mut output: *mut c_char = ptr::null_mut();
            let mut error: *mut c_char = ptr::null_mut();
            let mut code: c_int = 0;
            assert_eq!(
                du_execute(
                    ptr::null(),
                    ptr::null(),
                    ptr::null(),
                    ptr::null(),
                    &mut output,
                    &mut error,
                    &mut code
                ),
                -1
            );
            assert_eq!(
                du_execute(
                    to_c_str!("blah blah").unwrap().as_ptr(),
                    ptr::null(),
                    ptr::null(),
                    ptr::null(),
                    &mut output,
                    &mut error,
                    &mut code
                ),
                -2
            );
            let args: [*const c_char; 2] =
                [CString::new("My test").unwrap().into_raw(), ptr::null()];
            assert_eq!(
                du_execute(
                    to_c_str!("echo").unwrap().as_ptr(),
                    ptr::null(),
                    args.as_ptr(),
                    ptr::null(),
                    &mut output,
                    &mut error,
                    &mut code
                ),
                0
            );
            CString::from_raw(args[0] as *mut c_char);
            assert_eq!(code, 0);
            assert_eq!(from_c_str!(output).unwrap().trim(), "My test");
        }
    }

    #[test]
    fn open() {
        unsafe {
            assert_eq!(du_open(ptr::null()), -1);
            assert_eq!(du_open(to_c_str!("blah blah").unwrap().as_ptr()), -2);
        }
    }
}
