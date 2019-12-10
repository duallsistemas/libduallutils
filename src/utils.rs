#[doc(hidden)]
#[macro_export]
macro_rules! cs {
    ($rs:expr) => {
        std::ffi::CStr::from_ptr($rs).to_str()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! sc {
    ($cs:expr) => {
        std::ffi::CString::new($cs)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! copy {
    ($s:expr,$d:expr,$sz:expr) => {
        libc::memcpy($d as *mut libc::c_void, $s as *const libc::c_void, $sz)
    };
}
