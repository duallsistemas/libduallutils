use crypto::digest::Digest;
use crypto::md5::Md5;
use crypto::sha1::Sha1;
use libc::{c_char, c_int, size_t};

mod utils;

/// Retrieves library version as C-like string.
#[no_mangle]
pub unsafe extern "C" fn du_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), '\0').as_ptr() as *const c_char
}

/// Generates a MD5 from a given string.
///
/// # Arguments
///
/// * `[in] str` - Given C-like string.
/// * `[in,out] md5` - Generated MD5.
/// * `[in] size` - Size of the `str` string.
///
/// # Returns
///
/// * `0` - Success.
/// * `-1` - Invalid argument.
#[no_mangle]
pub unsafe extern "C" fn du_md5(str: *const c_char, md5: *mut c_char, size: size_t) -> c_int {
    if str.is_null() || md5.is_null() || size <= 0 {
        return -1;
    }
    let mut hasher = Md5::new();
    hasher.input_str(cs!(str).unwrap());
    let hash = sc!(hasher.result_str()).unwrap();
    let buf = hash.to_bytes_with_nul();
    let mut buf_size = size;
    if buf_size > buf.len() {
        buf_size = buf.len()
    }
    copy!(buf.as_ptr(), md5, buf_size);
    0
}

/// Generates a SHA-1 from a given string.
///
/// # Arguments
///
/// * `[in] str` - Given C-like string.
/// * `[in,out] sha1` - Generated SHA-1.
/// * `[in] size` - Size of the `str` string.
///
/// # Returns
///
/// * `0` - Success.
/// * `-1` - Invalid argument.
#[no_mangle]
pub unsafe extern "C" fn du_sha1(str: *const c_char, sha1: *mut c_char, size: size_t) -> c_int {
    if str.is_null() || sha1.is_null() || size <= 0 {
        return -1;
    }
    let mut hasher = Sha1::new();
    hasher.input_str(cs!(str).unwrap());
    let hash = sc!(hasher.result_str()).unwrap();
    let buf = hash.to_bytes_with_nul();
    let mut buf_size = size;
    if buf_size > buf.len() {
        buf_size = buf.len()
    }
    copy!(buf.as_ptr(), sha1, buf_size);
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version() {
        unsafe {
            assert_eq!(cs!(du_version()).unwrap(), env!("CARGO_PKG_VERSION"));
        }
    }

    #[test]
    fn md5() {
        unsafe {
            let hash: [c_char; 33] = [0; 33];
            assert_eq!(
                du_md5(std::ptr::null(), hash.as_ptr() as *mut c_char, 33),
                -1
            );
            assert_eq!(
                du_md5(sc!("abc123").unwrap().as_ptr(), std::ptr::null_mut(), 33),
                -1
            );
            assert_eq!(
                du_md5(
                    sc!("abc123").unwrap().as_ptr(),
                    hash.as_ptr() as *mut c_char,
                    0
                ),
                -1
            );
            assert_eq!(
                du_md5(
                    sc!("abc123").unwrap().as_ptr(),
                    hash.as_ptr() as *mut c_char,
                    hash.len()
                ),
                0
            );
            assert_eq!(
                cs!(hash.as_ptr()).unwrap(),
                "e99a18c428cb38d5f260853678922e03"
            );
        }
    }

    #[test]
    fn sha1() {
        unsafe {
            let hash: [c_char; 41] = [0; 41];
            assert_eq!(
                du_sha1(std::ptr::null(), hash.as_ptr() as *mut c_char, 41),
                -1
            );
            assert_eq!(
                du_sha1(sc!("abc123").unwrap().as_ptr(), std::ptr::null_mut(), 41),
                -1
            );
            assert_eq!(
                du_sha1(
                    sc!("abc123").unwrap().as_ptr(),
                    hash.as_ptr() as *mut c_char,
                    0
                ),
                -1
            );
            assert_eq!(
                du_sha1(
                    sc!("abc123").unwrap().as_ptr(),
                    hash.as_ptr() as *mut c_char,
                    hash.len()
                ),
                0
            );
            assert_eq!(
                cs!(hash.as_ptr()).unwrap(),
                "6367c48dd193d56ea7b0baad25b19455e529f5ee"
            );
        }
    }
}
