#[doc(hidden)]
#[macro_export]
macro_rules! from_c_str {
    ($pointer:expr) => {
        std::ffi::CStr::from_ptr($pointer).to_str()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! to_c_str {
    ($string:expr) => {
        std::ffi::CString::new($string)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! copy_memory {
    ($src:expr,$dest:expr,$size:expr) => {
        libc::memcpy(
            $dest as *mut libc::c_void,
            $src as *const libc::c_void,
            $size,
        )
    };
}
