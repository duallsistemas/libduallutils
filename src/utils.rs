use crate::DU_LOCKKEY::{
    self, {DU_LK_CAPSLOCK, DU_LK_NUMLOCK, DU_LK_SCROLLLOCK},
};
use libc::{c_char, size_t};
use lock_keys::LockKeys::{
    self, {CapitalLock, NumberLock, ScrollingLock},
};

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
            if arg.is_null() {
                break;
            }
            array.push(checked_str!(arg));
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
        // Falha fechado: nunca entregar string truncada sem terminador NUL.
        if $size < buf.len() {
            if $size > 0 {
                *$dest = 0;
            }
            return -1;
        }
        copy!(buf.as_ptr(), $dest, buf.len());
    };
}

impl From<LockKeys> for DU_LOCKKEY {
    fn from(key: LockKeys) -> Self {
        match key {
            CapitalLock => DU_LK_CAPSLOCK,
            NumberLock => DU_LK_NUMLOCK,
            ScrollingLock => DU_LK_SCROLLLOCK,
        }
    }
}

impl From<DU_LOCKKEY> for LockKeys {
    fn from(key: DU_LOCKKEY) -> Self {
        match key {
            DU_LK_CAPSLOCK => CapitalLock,
            DU_LK_NUMLOCK => NumberLock,
            DU_LK_SCROLLLOCK => ScrollingLock,
        }
    }
}

/// Valida UTF-8 antes de atravessar a interface C; ponteiro valido e NUL sao pre-condicoes.
#[doc(hidden)]
#[macro_export]
macro_rules! checked_str {
    ($value:expr) => {
        match from_c_str!($value) {
            Ok(value) => value,
            Err(_) => return -1,
        }
    };
}

/// Copia mensagem de erro do SO truncando em fronteira de caractere UTF-8, sempre
/// com terminador NUL. Diferente de hashes, mensagem truncada continua util.
pub unsafe fn copy_error_msg(msg: &str, dest: *mut c_char, size: size_t) {
    if dest.is_null() || size == 0 {
        return;
    }
    let mut len = msg.len().min(size - 1);
    while !msg.is_char_boundary(len) {
        len -= 1;
    }
    std::ptr::copy_nonoverlapping(msg.as_ptr(), dest as *mut u8, len);
    *dest.add(len) = 0;
}
