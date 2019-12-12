#[doc(hidden)]
#[macro_export]
macro_rules! from_c_str {
    ($c_str:expr) => {
        std::ffi::CStr::from_ptr($c_str).to_str()
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
macro_rules! from_c_array {
    ($array:expr) => {{
        let mut array = Vec::new();
        for i in 0.. {
            let arg = *($array.offset(i));
            if arg == ptr::null() {
                break;
            }
            array.push(from_c_str!(arg).unwrap());
        }
        array
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! copy {
    ($src:expr,$dest:expr,$size:expr) => {
        libc::memcpy(
            $dest as *mut libc::c_void,
            $src as *const libc::c_void,
            $size,
        )
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! copy_c_str {
    ($src:expr,$dest:expr,$size:expr) => {
        let buf = $src.to_bytes_with_nul();
        let mut buf_size = $size;
        if buf_size > buf.len() {
            buf_size = buf.len()
        }
        copy!(buf.as_ptr(), $dest, buf_size);
    };
}
