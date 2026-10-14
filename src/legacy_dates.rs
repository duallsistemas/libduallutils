//! Independent implementation of Delphi 13 legacy file-date/read-only contracts.
#[cfg(test)]
use std::ptr;
use std::slice;

fn path_units(path: &[u16]) -> &[u16] {
    &path[..path.iter().position(|c| *c == 0).unwrap_or(path.len())]
}
#[cfg(windows)]
fn wide(path: &[u16]) -> Vec<u16> {
    path_units(path).iter().copied().chain(Some(0)).collect()
}
#[cfg(target_os = "linux")]
fn utf8(path: &[u16]) -> std::ffi::CString {
    std::ffi::CString::new(String::from_utf16_lossy(path_units(path))).expect("NUL stripped")
}

#[cfg(windows)]
mod platform {
    use super::wide;
    use std::ptr;
    use winapi::shared::minwindef::FILETIME;
    use winapi::um::{
        errhandlingapi::GetLastError,
        fileapi::{
            CreateFileW, FileTimeToLocalFileTime, FindClose, FindFirstFileW, GetFileAttributesExW,
            GetFileAttributesW, GetFinalPathNameByHandleW, LocalFileTimeToFileTime, SetFileTime,
            OPEN_EXISTING, WIN32_FILE_ATTRIBUTE_DATA,
        },
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        minwinbase::{GetFileExInfoStandard, WIN32_FIND_DATAW},
        winbase::{DosDateTimeToFileTime, FILE_FLAG_BACKUP_SEMANTICS},
        winnt::{
            FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_READONLY,
            FILE_ATTRIBUTE_REPARSE_POINT, FILE_SHARE_READ, GENERIC_READ, GENERIC_WRITE,
        },
    };

    struct OwnedHandle(winapi::shared::ntdef::HANDLE);
    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }

    // Resolving a link must itself obey the RTL read/share flags. Failure leaves
    // the link's own attributes in place, including dangling or locked targets.
    unsafe fn target(path: &[u16], attrs: u32) -> Option<Vec<u16>> {
        let flags = if attrs & FILE_ATTRIBUTE_DIRECTORY != 0 {
            FILE_FLAG_BACKUP_SEMANTICS
        } else {
            0
        };
        let handle = CreateFileW(
            path.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ,
            ptr::null_mut(),
            OPEN_EXISTING,
            flags,
            ptr::null_mut(),
        );
        if handle == INVALID_HANDLE_VALUE {
            return None;
        }
        let _owner = OwnedHandle(handle);
        let count = GetFinalPathNameByHandleW(handle, ptr::null_mut(), 0, 0);
        let result = if count == 0 {
            None
        } else {
            let mut result = vec![0; count as usize + 1];
            let copied =
                GetFinalPathNameByHandleW(handle, result.as_mut_ptr(), result.len() as u32, 0);
            if copied == 0 || copied as usize >= result.len() {
                None
            } else {
                result.truncate(copied as usize);
                result.push(0);
                Some(result)
            }
        };
        result
    }
    unsafe fn attrs(path: &[u16], data: &mut WIN32_FILE_ATTRIBUTE_DATA) -> bool {
        GetFileAttributesExW(
            path.as_ptr(),
            GetFileExInfoStandard,
            data as *mut _ as *mut _,
        ) != 0
    }
    pub unsafe fn age(path: &[u16]) -> Result<i64, i32> {
        let path = wide(path);
        let mut data: WIN32_FILE_ATTRIBUTE_DATA = std::mem::zeroed();
        let mut success = attrs(&path, &mut data);
        if success && data.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            if let Some(target) = target(&path, data.dwFileAttributes) {
                success = attrs(&target, &mut data);
            }
        }
        if !success {
            let error = GetLastError();
            if error != 32 && error != 33 {
                return Err(-2);
            }
            let mut found: WIN32_FIND_DATAW = std::mem::zeroed();
            let handle = FindFirstFileW(path.as_ptr(), &mut found);
            if handle == INVALID_HANDLE_VALUE {
                return Err(-2);
            }
            FindClose(handle);
            data.dwFileAttributes = found.dwFileAttributes;
            data.ftLastWriteTime = found.ftLastWriteTime;
        }
        if data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
            return Err(-2);
        }
        let mut local: FILETIME = std::mem::zeroed();
        if FileTimeToLocalFileTime(&data.ftLastWriteTime, &mut local) == 0 {
            return Err(-2);
        }
        Ok(((u64::from(local.dwHighDateTime) << 32) | u64::from(local.dwLowDateTime)) as i64)
    }
    pub unsafe fn set_date(path: &[u16], packed: i64) -> Result<(), i32> {
        let path = wide(path);
        let handle = CreateFileW(
            path.as_ptr(),
            GENERIC_WRITE,
            0,
            ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            ptr::null_mut(),
        );
        if handle == INVALID_HANDLE_VALUE {
            return Err(-2);
        }
        let mut local: FILETIME = std::mem::zeroed();
        let mut utc: FILETIME = std::mem::zeroed();
        let success =
            DosDateTimeToFileTime((packed as u32 >> 16) as u16, packed as u16, &mut local) != 0
                && LocalFileTimeToFileTime(&local, &mut utc) != 0
                && SetFileTime(handle, ptr::null(), ptr::null(), &utc) != 0;
        CloseHandle(handle);
        if success {
            Ok(())
        } else {
            Err(-2)
        }
    }
    pub unsafe fn readonly(path: &[u16]) -> bool {
        let path = wide(path);
        let mut attrs = GetFileAttributesW(path.as_ptr());
        if attrs == u32::MAX {
            return false;
        }
        if attrs & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            if let Some(target) = target(&path, attrs) {
                attrs = GetFileAttributesW(target.as_ptr());
            }
        }
        attrs & FILE_ATTRIBUTE_READONLY != 0
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::utf8;
    extern "C" {
        fn euidaccess(path: *const libc::c_char, mode: libc::c_int) -> libc::c_int;
    }
    pub unsafe fn age(path: &[u16]) -> Result<i64, i32> {
        let path = utf8(path);
        let mut stat: libc::stat = std::mem::zeroed();
        if libc::stat(path.as_ptr(), &mut stat) != 0 {
            Err(-2)
        } else {
            Ok(stat.st_mtime as i64)
        }
    }
    pub unsafe fn set_date(path: &[u16], packed: i64) -> Result<(), i32> {
        let path = utf8(path);
        let times = libc::utimbuf {
            actime: packed as libc::time_t,
            modtime: packed as libc::time_t,
        };
        if libc::utime(path.as_ptr(), &times) == 0 {
            Ok(())
        } else {
            Err(-2)
        }
    }
    pub unsafe fn readonly(path: &[u16]) -> bool {
        let path = utf8(path);
        euidaccess(path.as_ptr(), libc::R_OK) == 0 && euidaccess(path.as_ptr(), libc::W_OK) != 0
    }
}

fn wait_loop(mut readonly: impl FnMut() -> bool, mut sleep: impl FnMut()) -> bool {
    let mut count = 0;
    while readonly() {
        count += 1;
        sleep();
        if count > 100 {
            return false;
        }
    }
    true
}
unsafe fn units<'a>(path: *const u16, len: usize) -> Result<&'a [u16], i32> {
    if (path.is_null() && len != 0) || len > isize::MAX as usize / 2 {
        return Err(-1);
    }
    Ok(if len == 0 {
        &[]
    } else {
        slice::from_raw_parts(path, len)
    })
}
/// Raw local FILETIME on Windows; stat.st_mtime seconds on Linux.
/// Output is untouched on failure: 0 success, -1 argument, -2 OS/conversion, -3 panic.
#[no_mangle]
pub unsafe extern "C" fn du_file_age_legacy(
    path: *const u16,
    len: usize,
    out_raw: *mut i64,
) -> i32 {
    if out_raw.is_null() {
        return -1;
    }
    let path = match units(path, len) {
        Ok(v) => v,
        Err(code) => return code,
    };
    match std::panic::catch_unwind(|| platform::age(path)) {
        Ok(Ok(raw)) => {
            *out_raw = raw;
            0
        }
        Ok(Err(code)) => code,
        Err(_) => -3,
    }
}
/// Windows packed DOS date/time in low32 bits; Linux signed i64 seconds sets atime+mtime.
#[no_mangle]
pub unsafe extern "C" fn du_set_file_date_legacy(path: *const u16, len: usize, packed: i64) -> i32 {
    let path = match units(path, len) {
        Ok(v) => v,
        Err(code) => return code,
    };
    match std::panic::catch_unwind(|| platform::set_date(path, packed)) {
        Ok(Ok(())) => 0,
        Ok(Err(code)) => code,
        Err(_) => -3,
    }
}
/// Read-only attribute/access loop, not an exclusive-open probe. No existence guard.
/// Caller receives 1 writable or 0 after 101 sleeps (100ms each); preserves output on errors.
#[no_mangle]
pub unsafe extern "C" fn du_wait_writable_legacy(
    path: *const u16,
    len: usize,
    result: *mut u8,
) -> i32 {
    if result.is_null() {
        return -1;
    }
    let path = match units(path, len) {
        Ok(v) => v,
        Err(code) => return code,
    };
    match std::panic::catch_unwind(|| {
        wait_loop(
            || platform::readonly(path),
            || std::thread::sleep(std::time::Duration::from_millis(100)),
        )
    }) {
        Ok(value) => {
            *result = u8::from(value);
            0
        }
        Err(_) => -3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fake_wait_checks_exact_timeout_and_transitions() {
        let mut probes = 0;
        let mut sleeps = 0;
        assert!(!wait_loop(
            || {
                probes += 1;
                true
            },
            || sleeps += 1
        ));
        assert_eq!(probes, 101);
        assert_eq!(sleeps, 101);
        let mut probes = 0;
        let mut sleeps = 0;
        assert!(wait_loop(
            || {
                probes += 1;
                probes <= 2
            },
            || sleeps += 1
        ));
        assert_eq!(probes, 3);
        assert_eq!(sleeps, 2);
        assert!(wait_loop(|| false, || panic!("unexpected sleep")));
    }
    #[test]
    fn unicode_dates_locked_missing_and_directories() {
        unsafe {
            let folder = std::env::temp_dir().join(format!("duall-dates-{}", std::process::id()));
            std::fs::create_dir(&folder).unwrap();
            let file = folder.join("date-\u{e7} space.bin");
            std::fs::write(&file, b"date").unwrap();
            let path: Vec<u16> = file.to_str().unwrap().encode_utf16().collect();
            let mut raw = 123;
            #[cfg(windows)]
            let packed =
                (((2022 - 1980) << 25) | (6 << 21) | (15 << 16) | (12 << 11) | (34 << 5) | 28)
                    as i64;
            #[cfg(target_os = "linux")]
            let packed = 1_650_000_000i64;
            assert_eq!(
                du_set_file_date_legacy(path.as_ptr(), path.len(), packed),
                0
            );
            assert_eq!(du_file_age_legacy(path.as_ptr(), path.len(), &mut raw), 0);
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                use winapi::shared::minwindef::FILETIME;
                use winapi::um::winbase::DosDateTimeToFileTime;
                let mut expected: FILETIME = std::mem::zeroed();
                assert_ne!(
                    DosDateTimeToFileTime(
                        (packed as u32 >> 16) as u16,
                        packed as u16,
                        &mut expected
                    ),
                    0
                );
                assert_eq!(
                    raw,
                    ((u64::from(expected.dwHighDateTime) << 32) | u64::from(expected.dwLowDateTime))
                        as i64
                );
                let locked = std::fs::OpenOptions::new()
                    .read(true)
                    .share_mode(0)
                    .open(&file)
                    .unwrap();
                assert_eq!(du_file_age_legacy(path.as_ptr(), path.len(), &mut raw), 0);
                assert_eq!(
                    du_set_file_date_legacy(path.as_ptr(), path.len(), packed),
                    -2
                );
                drop(locked);
                let mut permissions = std::fs::metadata(&file).unwrap().permissions();
                permissions.set_readonly(true);
                std::fs::set_permissions(&file, permissions).unwrap();
                assert!(platform::readonly(&path));
                let mut permissions = std::fs::metadata(&file).unwrap().permissions();
                permissions.set_readonly(false);
                std::fs::set_permissions(&file, permissions).unwrap();
                assert!(!platform::readonly(&path));
            }
            #[cfg(target_os = "linux")]
            {
                use std::os::unix::fs::{MetadataExt, PermissionsExt};
                assert_eq!(raw, packed);
                assert_eq!(std::fs::metadata(&file).unwrap().atime(), packed);
                std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o400)).unwrap();
                assert_eq!(platform::readonly(&path), libc::geteuid() != 0);
                std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
                assert!(!platform::readonly(&path));
            }
            let directory: Vec<u16> = folder.to_str().unwrap().encode_utf16().collect();
            raw = 444;
            #[cfg(windows)]
            {
                assert_eq!(
                    du_file_age_legacy(directory.as_ptr(), directory.len(), &mut raw),
                    -2
                );
                assert_eq!(raw, 444);
            }
            #[cfg(target_os = "linux")]
            assert_eq!(
                du_file_age_legacy(directory.as_ptr(), directory.len(), &mut raw),
                0
            );
            std::fs::remove_file(&file).unwrap();
            raw = 999;
            assert_eq!(du_file_age_legacy(path.as_ptr(), path.len(), &mut raw), -2);
            assert_eq!(raw, 999);
            assert!(!platform::readonly(&path));
            let mut result = 77;
            assert_eq!(
                du_wait_writable_legacy(path.as_ptr(), path.len(), &mut result),
                0
            );
            assert_eq!(result, 1);
            assert_eq!(du_file_age_legacy(ptr::null(), 1, &mut raw), -1);
            assert_eq!(raw, 999);
            std::fs::remove_dir(folder).unwrap();
        }
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn linux_setter_preserves_64bit_and_negative_epoch() {
        use std::os::unix::fs::MetadataExt;
        let path = std::env::temp_dir().join(format!("duall-date64-{}.bin", std::process::id()));
        std::fs::write(&path, b"date").unwrap();
        let units: Vec<u16> = path.to_str().unwrap().encode_utf16().collect();
        for epoch in [2_160_232_497i64, 4_355_481_600i64, -1i64] {
            let mut raw = 0;
            unsafe {
                assert_eq!(
                    du_set_file_date_legacy(units.as_ptr(), units.len(), epoch),
                    0
                );
                assert_eq!(du_file_age_legacy(units.as_ptr(), units.len(), &mut raw), 0);
            }
            assert_eq!(raw, epoch);
            let metadata = std::fs::metadata(&path).unwrap();
            assert_eq!(metadata.mtime(), epoch);
            assert_eq!(metadata.atime(), epoch);
        }
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn symbolic_link_dates_follow_accessible_target() {
        unsafe {
            let folder =
                std::env::temp_dir().join(format!("duall-date-links-{}", std::process::id()));
            std::fs::create_dir(&folder).unwrap();
            let target = folder.join("target.bin");
            let link = folder.join("link.bin");
            std::fs::write(&target, b"date").unwrap();
            #[cfg(windows)]
            let created = std::os::windows::fs::symlink_file(&target, &link);
            #[cfg(target_os = "linux")]
            let created = std::os::unix::fs::symlink("target.bin", &link);
            if let Err(error) = created {
                println!("Windows symlink test unsupported: {}", error);
                std::fs::remove_file(target).unwrap();
                std::fs::remove_dir(folder).unwrap();
                return;
            }
            let target_units: Vec<u16> = target.to_str().unwrap().encode_utf16().collect();
            let link_units: Vec<u16> = link.to_str().unwrap().encode_utf16().collect();
            let mut target_raw = 0;
            let mut link_raw = 0;
            assert_eq!(
                du_file_age_legacy(target_units.as_ptr(), target_units.len(), &mut target_raw),
                0
            );
            assert_eq!(
                du_file_age_legacy(link_units.as_ptr(), link_units.len(), &mut link_raw),
                0
            );
            assert_eq!(link_raw, target_raw);
            std::fs::remove_file(link).unwrap();
            std::fs::remove_file(target).unwrap();
            std::fs::remove_dir(folder).unwrap();
        }
    }
}
