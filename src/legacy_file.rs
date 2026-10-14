//! Legacy dUtils filesystem calls. No existence precheck: it stays in Delphi.
#[cfg(any(windows, test))]
use std::ptr;
use std::slice;

#[cfg(windows)]
unsafe fn apply(op: u32, path: &[u16], value: *mut i64) -> i32 {
    use winapi::um::errhandlingapi::SetLastError;
    use winapi::um::fileapi::{
        CreateFileW, DeleteFileW, GetFileAttributesExW, OPEN_EXISTING, WIN32_FILE_ATTRIBUTE_DATA,
    };
    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
    use winapi::um::minwinbase::GetFileExInfoStandard;
    use winapi::um::winnt::{FILE_ATTRIBUTE_NORMAL, GENERIC_READ};
    let path: Vec<u16> = path.iter().copied().chain(Some(0)).collect();
    match op {
        1 => {
            SetLastError(0);
            let mut data: WIN32_FILE_ATTRIBUTE_DATA = std::mem::zeroed();
            if GetFileAttributesExW(
                path.as_ptr(),
                GetFileExInfoStandard,
                &mut data as *mut _ as *mut _,
            ) == 0
            {
                return -2;
            }
            *value = ((u64::from(data.nFileSizeHigh) << 32) | u64::from(data.nFileSizeLow)) as i64;
            0
        }
        2 => {
            if DeleteFileW(path.as_ptr()) != 0 {
                0
            } else {
                -2
            }
        }
        3 => {
            let handle = CreateFileW(
                path.as_ptr(),
                GENERIC_READ,
                0,
                ptr::null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                ptr::null_mut(),
            );
            *value = if handle == INVALID_HANDLE_VALUE {
                1
            } else {
                CloseHandle(handle);
                0
            };
            0
        }
        _ => -1,
    }
}

#[cfg(target_os = "linux")]
unsafe fn apply(op: u32, path: &[u16], value: *mut i64) -> i32 {
    // The Delphi UTF-8 marshaller replaces malformed UTF-16; native calls stop at NUL.
    let count = path.iter().position(|c| *c == 0).unwrap_or(path.len());
    let path = String::from_utf16_lossy(&path[..count]);
    let path = std::ffi::CString::new(path).expect("NUL stripped before conversion");
    match op {
        1 => {
            let mut data: libc::stat = std::mem::zeroed();
            if libc::stat(path.as_ptr(), &mut data) != 0
                || data.st_mode & libc::S_IFMT != libc::S_IFREG
            {
                return -2;
            }
            *value = data.st_size as i64;
            0
        }
        2 => {
            if libc::unlink(path.as_ptr()) == 0 {
                0
            } else {
                -2
            }
        }
        3 => {
            let fd = libc::open(path.as_ptr(), libc::O_RDONLY);
            *value = if fd < 0 {
                1
            } else {
                libc::close(fd);
                0
            };
            0
        }
        _ => -1,
    }
}

/// op1 metadata size, op2 DeleteFile/unlink, op3 exclusive-read open probe.
/// 0 success; -1 invalid argument/op; -2 OS failure; -3 caught panic.
/// Value is required for ops1/3 and unchanged on failure; unused for op2.
/// Input must point to valid aligned UTF-16 units; an embedded NUL ends the path.
#[no_mangle]
pub unsafe extern "C" fn du_legacy_file(
    op: u32,
    path: *const u16,
    len: usize,
    value: *mut i64,
) -> i32 {
    if !(1..=3).contains(&op)
        || (path.is_null() && len != 0)
        || len > isize::MAX as usize / 2
        || (op != 2 && value.is_null())
    {
        return -1;
    }
    let result = std::panic::catch_unwind(|| {
        let path = if len == 0 {
            &[][..]
        } else {
            slice::from_raw_parts(path, len)
        };
        apply(op, path, value)
    });
    match result {
        Ok(code) => code,
        Err(_) => -3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn size_probe_delete_and_missing_preserve_output() {
        unsafe {
            let folder = std::env::temp_dir().join(format!("duall-file-{}", std::process::id()));
            std::fs::create_dir_all(&folder).unwrap();
            let file = folder.join("unicode-\u{e7} space.bin");
            std::fs::write(&file, b"hello").unwrap();
            let path: Vec<u16> = file.to_str().unwrap().encode_utf16().collect();
            let mut size = 99;
            assert_eq!(du_legacy_file(1, path.as_ptr(), path.len(), &mut size), 0);
            assert_eq!(size, 5);
            assert_eq!(du_legacy_file(3, path.as_ptr(), path.len(), &mut size), 0);
            assert_eq!(size, 0);
            let mut nul = path.clone();
            nul.extend([0, 65, 66]);
            assert_eq!(du_legacy_file(1, nul.as_ptr(), nul.len(), &mut size), 0);
            assert_eq!(size, 5);
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                let locked = std::fs::OpenOptions::new()
                    .read(true)
                    .share_mode(0)
                    .open(&file)
                    .unwrap();
                assert_eq!(du_legacy_file(3, path.as_ptr(), path.len(), &mut size), 0);
                assert_eq!(size, 1);
                assert_eq!(
                    du_legacy_file(2, path.as_ptr(), path.len(), ptr::null_mut()),
                    -2
                );
                drop(locked);
            }
            assert_eq!(
                du_legacy_file(2, path.as_ptr(), path.len(), ptr::null_mut()),
                0
            );
            size = 123;
            assert_eq!(du_legacy_file(1, path.as_ptr(), path.len(), &mut size), -2);
            assert_eq!(size, 123);
            assert_eq!(
                du_legacy_file(2, path.as_ptr(), path.len(), ptr::null_mut()),
                -2
            );
            let dir: Vec<u16> = folder.to_str().unwrap().encode_utf16().collect();
            #[cfg(windows)]
            assert_eq!(du_legacy_file(1, dir.as_ptr(), dir.len(), &mut size), 0);
            #[cfg(target_os = "linux")]
            assert_eq!(du_legacy_file(1, dir.as_ptr(), dir.len(), &mut size), -2);
            assert_eq!(
                du_legacy_file(2, dir.as_ptr(), dir.len(), ptr::null_mut()),
                -2
            );
            std::fs::remove_dir(folder).unwrap();
        }
    }
    #[test]
    fn invalid_arguments() {
        unsafe {
            let mut value = 77;
            assert_eq!(du_legacy_file(99, ptr::null(), 0, &mut value), -1);
            assert_eq!(du_legacy_file(1, ptr::null(), 1, &mut value), -1);
            assert_eq!(du_legacy_file(1, ptr::null(), 0, ptr::null_mut()), -1);
            assert_eq!(value, 77);
        }
    }
}
