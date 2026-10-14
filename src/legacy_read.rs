//! Single-snapshot legacy file read; callers own text decoding.
use libc::c_void;
use std::convert::TryFrom;
use std::{
    fs::File,
    io::{Read, Write},
    ptr, slice,
};

#[cfg(windows)]
unsafe fn open(path: &[u16]) -> std::io::Result<File> {
    use std::os::windows::io::FromRawHandle;
    use winapi::um::{
        fileapi::{CreateFileW, OPEN_EXISTING},
        handleapi::INVALID_HANDLE_VALUE,
        winnt::{FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, GENERIC_READ},
    };
    let path: Vec<u16> = path.iter().copied().chain(Some(0)).collect();
    let handle = CreateFileW(
        path.as_ptr(),
        GENERIC_READ,
        FILE_SHARE_READ,
        ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        ptr::null_mut(),
    );
    if handle == INVALID_HANDLE_VALUE {
        return Err(std::io::Error::last_os_error());
    }
    Ok(File::from_raw_handle(handle as *mut _))
}

#[cfg(target_os = "linux")]
unsafe fn open(path: &[u16]) -> std::io::Result<File> {
    use std::os::unix::io::FromRawFd;
    let end = path.iter().position(|c| *c == 0).unwrap_or(path.len());
    let path =
        std::ffi::CString::new(String::from_utf16_lossy(&path[..end])).expect("NUL stripped");
    let fd = libc::open(path.as_ptr(), libc::O_RDONLY);
    if fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(File::from_raw_fd(fd))
}

fn read_snapshot(mut file: File) -> Result<Vec<u8>, (i32, i32)> {
    let size = file
        .metadata()
        .map_err(|e| (-2, e.raw_os_error().unwrap_or(0)))?
        .len();
    read_size(&mut file, size)
}

fn read_size(file: &mut File, size: u64) -> Result<Vec<u8>, (i32, i32)> {
    let size = usize::try_from(size).map_err(|_| (-3, 0))?;
    let mut data = Vec::new();
    data.try_reserve_exact(size).map_err(|_| (-4, 0))?;
    data.resize(size, 0);
    file.read_exact(&mut data)
        .map_err(|e| (-3, e.raw_os_error().unwrap_or(0)))?;
    Ok(data)
}

/// Returns borrowed data tied to out_owner; release with du_read_file_dispose.
/// Empty success also returns an owner. Failures clear outputs and preserve OS code.
/// 0 success, -1 invalid arguments, -2 open/metadata, -3 read, -4 panic/allocation.
/// Pointer storage must be valid, aligned and non-overlapping.
#[no_mangle]
pub unsafe extern "C" fn du_read_file_legacy(
    path: *const u16,
    len: usize,
    out_data: *mut *const u8,
    out_len: *mut usize,
    out_owner: *mut *mut c_void,
    out_os_error: *mut i32,
) -> i32 {
    if !out_data.is_null() {
        *out_data = ptr::null();
    }
    if !out_len.is_null() {
        *out_len = 0;
    }
    if !out_owner.is_null() {
        *out_owner = ptr::null_mut();
    }
    if !out_os_error.is_null() {
        *out_os_error = 0;
    }
    if out_data.is_null()
        || out_len.is_null()
        || out_owner.is_null()
        || out_os_error.is_null()
        || (path.is_null() && len != 0)
        || len > isize::MAX as usize / 2
    {
        return -1;
    }
    let result = std::panic::catch_unwind(|| {
        let units = if len == 0 {
            &[][..]
        } else {
            slice::from_raw_parts(path, len)
        };
        let file = open(units).map_err(|e| (-2, e.raw_os_error().unwrap_or(0)))?;
        read_snapshot(file)
    });
    match result {
        Ok(Ok(data)) => {
            let owner = Box::new(data);
            *out_data = owner.as_ptr();
            *out_len = owner.len();
            *out_owner = Box::into_raw(owner) as *mut c_void;
            0
        }
        Ok(Err((status, code))) => {
            *out_os_error = code;
            status
        }
        Err(_) => -4,
    }
}

#[cfg(windows)]
unsafe fn borrowed(handle: usize) -> std::mem::ManuallyDrop<File> {
    use std::os::windows::io::FromRawHandle;
    std::mem::ManuallyDrop::new(File::from_raw_handle(handle as *mut _))
}
#[cfg(target_os = "linux")]
unsafe fn borrowed(handle: usize) -> std::mem::ManuallyDrop<File> {
    use std::os::unix::io::FromRawFd;
    std::mem::ManuallyDrop::new(File::from_raw_fd(handle as i32))
}
fn invalid_handle(handle: usize) -> bool {
    #[cfg(windows)]
    {
        handle == 0 || handle == usize::MAX
    }
    #[cfg(target_os = "linux")]
    {
        handle > i32::MAX as usize
    }
}

/// Reads exactly the caller's initial size using a BORROWED native handle.
/// The caller closes the handle. Outputs use du_read_file_dispose ownership.
#[no_mangle]
pub unsafe extern "C" fn du_read_handle_legacy(
    handle: usize,
    size: u64,
    out_data: *mut *const u8,
    out_len: *mut usize,
    out_owner: *mut *mut c_void,
    out_os_error: *mut i32,
) -> i32 {
    if !out_data.is_null() {
        *out_data = ptr::null();
    }
    if !out_len.is_null() {
        *out_len = 0;
    }
    if !out_owner.is_null() {
        *out_owner = ptr::null_mut();
    }
    if !out_os_error.is_null() {
        *out_os_error = 0;
    }
    if invalid_handle(handle)
        || out_data.is_null()
        || out_len.is_null()
        || out_owner.is_null()
        || out_os_error.is_null()
    {
        return -1;
    }
    let result = std::panic::catch_unwind(|| {
        let mut file = borrowed(handle);
        read_size(&mut file, size)
    });
    match result {
        Ok(Ok(data)) => {
            let owner = Box::new(data);
            *out_data = owner.as_ptr();
            *out_len = owner.len();
            *out_owner = Box::into_raw(owner) as *mut c_void;
            0
        }
        Ok(Err((status, code))) => {
            *out_os_error = code;
            status
        }
        Err(_) => -4,
    }
}

/// Writes bytes at the current position of a BORROWED native handle.
/// The caller creates/seeks/closes the file; this export never closes it.
#[no_mangle]
pub unsafe extern "C" fn du_write_handle_legacy(
    handle: usize,
    data: *const u8,
    size: usize,
    out_os_error: *mut i32,
) -> i32 {
    if !out_os_error.is_null() {
        *out_os_error = 0;
    }
    if invalid_handle(handle)
        || out_os_error.is_null()
        || (data.is_null() && size != 0)
        || size > isize::MAX as usize
    {
        return -1;
    }
    let result = std::panic::catch_unwind(|| {
        let data = if size == 0 {
            &[][..]
        } else {
            slice::from_raw_parts(data, size)
        };
        let mut file = borrowed(handle);
        file.write_all(data)
    });
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => {
            *out_os_error = error.raw_os_error().unwrap_or(0);
            -3
        }
        Err(_) => -4,
    }
}

/// Owner must be null or an unfreed pointer returned by du_read_file_legacy.
#[no_mangle]
pub unsafe extern "C" fn du_read_file_dispose(owner: *mut c_void) {
    if !owner.is_null() {
        drop(Box::from_raw(owner as *mut Vec<u8>));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn borrowed_handles_preserve_owner_position_and_snapshot_size() {
        unsafe {
            use std::io::{Seek, SeekFrom};
            #[cfg(target_os = "linux")]
            use std::os::unix::io::AsRawFd;
            #[cfg(windows)]
            use std::os::windows::io::AsRawHandle;
            let path =
                std::env::temp_dir().join(format!("duall-handle-{}.bin", std::process::id()));
            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap();
            #[cfg(windows)]
            let handle = file.as_raw_handle() as usize;
            #[cfg(target_os = "linux")]
            let handle = file.as_raw_fd() as usize;
            let mut code = 0;
            assert_eq!(
                du_write_handle_legacy(handle, b"abcdef".as_ptr(), 6, &mut code),
                0
            );
            file.seek(SeekFrom::Start(0)).unwrap();
            let mut data = ptr::null();
            let mut len = 0;
            let mut owner = ptr::null_mut();
            assert_eq!(
                du_read_handle_legacy(handle, 3, &mut data, &mut len, &mut owner, &mut code),
                0
            );
            assert_eq!(slice::from_raw_parts(data, len), b"abc");
            du_read_file_dispose(owner);
            assert_eq!(file.stream_position().unwrap(), 3);
            assert_eq!(
                du_read_handle_legacy(handle, 4, &mut data, &mut len, &mut owner, &mut code),
                -3
            );
            assert!(data.is_null());
            assert!(owner.is_null());
            assert_eq!(len, 0);
            file.seek(SeekFrom::Start(0)).unwrap();
            file.write_all(b"X").unwrap();
            drop(file);
            assert_eq!(std::fs::read(&path).unwrap(), b"Xbcdef");
            std::fs::remove_file(path).unwrap();
        }
    }
    #[test]
    fn bytes_empty_missing_sharing_and_dispose() {
        unsafe {
            let path =
                std::env::temp_dir().join(format!("duall read \u{e7} {}.bin", std::process::id()));
            let units: Vec<u16> = path.to_str().unwrap().encode_utf16().collect();
            let mut data = ptr::null();
            let mut len = 0;
            let mut owner = ptr::null_mut();
            let mut code = 0;
            let bytes: Vec<u8> = (0..=255).collect();
            std::fs::write(&path, &bytes).unwrap();
            assert_eq!(
                du_read_file_legacy(
                    units.as_ptr(),
                    units.len(),
                    &mut data,
                    &mut len,
                    &mut owner,
                    &mut code
                ),
                0
            );
            assert_eq!(slice::from_raw_parts(data, len), bytes);
            assert!(!owner.is_null());
            du_read_file_dispose(owner);
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                let locked = std::fs::OpenOptions::new()
                    .write(true)
                    .share_mode(3)
                    .open(&path)
                    .unwrap();
                assert_eq!(
                    du_read_file_legacy(
                        units.as_ptr(),
                        units.len(),
                        &mut data,
                        &mut len,
                        &mut owner,
                        &mut code
                    ),
                    -2
                );
                assert_eq!(code, 32);
                assert!(data.is_null());
                assert!(owner.is_null());
                assert_eq!(len, 0);
                drop(locked);
            }
            std::fs::write(&path, []).unwrap();
            assert_eq!(
                du_read_file_legacy(
                    units.as_ptr(),
                    units.len(),
                    &mut data,
                    &mut len,
                    &mut owner,
                    &mut code
                ),
                0
            );
            assert_eq!(len, 0);
            assert!(!owner.is_null());
            du_read_file_dispose(owner);
            std::fs::remove_file(&path).unwrap();
            assert_eq!(
                du_read_file_legacy(
                    units.as_ptr(),
                    units.len(),
                    &mut data,
                    &mut len,
                    &mut owner,
                    &mut code
                ),
                -2
            );
            assert!(code != 0);
            assert!(data.is_null());
            assert!(owner.is_null());
            assert_eq!(len, 0);
            du_read_file_dispose(ptr::null_mut());
            assert_eq!(
                du_read_file_legacy(ptr::null(), 1, &mut data, &mut len, &mut owner, &mut code),
                -1
            );
            assert_eq!(
                du_read_file_legacy(
                    ptr::null(),
                    0,
                    ptr::null_mut(),
                    &mut len,
                    &mut owner,
                    &mut code
                ),
                -1
            );
        }
    }
}
