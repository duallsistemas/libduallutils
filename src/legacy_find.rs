//! Streaming Delphi FindFirst/FindNext compatibility. Owners must originate
//! from open, must not be shared concurrently, and must be closed exactly once.
use libc::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

struct Record {
    name: Vec<u16>,
    mtime: i64,
    attrs: u32,
}

struct Owner {
    search: platform::Search,
    pending: Option<Record>,
}

unsafe fn wide<'a>(value: *const u16, len: usize) -> Result<&'a [u16], i32> {
    if len > isize::MAX as usize / 2 || (value.is_null() && len != 0) {
        return Err(-1);
    }
    Ok(if len == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(value, len)
    })
}

/// All non-null pointers must designate valid, suitably aligned storage.
#[no_mangle]
pub unsafe extern "C" fn du_find_open_legacy(
    mask: *const u16,
    masklen: usize,
    directory: *const u16,
    dirlen: usize,
    pattern: *const u16,
    patlen: usize,
    attrs: i32,
    out_owner: *mut *mut c_void,
) -> i32 {
    if out_owner.is_null() {
        return -1;
    }
    *out_owner = ptr::null_mut();
    catch_unwind(AssertUnwindSafe(|| {
        let result = (|| {
            let mut search = platform::Search::open(
                wide(mask, masklen)?,
                wide(directory, dirlen)?,
                wide(pattern, patlen)?,
                attrs,
            )?;
            let pending = search.next()?.ok_or(-2)?;
            Ok::<_, i32>(Owner {
                search,
                pending: Some(pending),
            })
        })();
        match result {
            Ok(owner) => {
                *out_owner = Box::into_raw(Box::new(owner)) as *mut c_void;
                0
            }
            Err(status) => status,
        }
    }))
    .unwrap_or(-3)
}

/// Query and insufficient-buffer calls retain the pending record. A successful
/// copy consumes it. Names contain exactly required UTF-16 units, without NUL.
#[no_mangle]
pub unsafe extern "C" fn du_find_next_legacy(
    owner: *mut c_void,
    name: *mut u16,
    cap: usize,
    required: *mut usize,
    mtime: *mut i64,
    fileattrs: *mut u32,
) -> i32 {
    if owner.is_null()
        || required.is_null()
        || mtime.is_null()
        || fileattrs.is_null()
        || cap > isize::MAX as usize / 2
        || (name.is_null() && cap != 0)
    {
        return -1;
    }
    *required = 0;
    catch_unwind(AssertUnwindSafe(|| {
        let owner = &mut *(owner as *mut Owner);
        if owner.pending.is_none() {
            match owner.search.next() {
                Ok(value) => owner.pending = value,
                Err(status) => return status,
            }
        }
        let record = match owner.pending.as_ref() {
            Some(value) => value,
            None => return 1,
        };
        *required = record.name.len();
        *mtime = record.mtime;
        *fileattrs = record.attrs;
        if name.is_null() && cap == 0 {
            return 0;
        }
        if cap < record.name.len() {
            return -1;
        }
        ptr::copy_nonoverlapping(record.name.as_ptr(), name, record.name.len());
        owner.pending = None;
        0
    }))
    .unwrap_or(-3)
}

#[no_mangle]
pub unsafe extern "C" fn du_find_close_legacy(owner: *mut c_void) {
    if !owner.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            drop(Box::from_raw(owner as *mut Owner))
        }));
    }
}

#[cfg(windows)]
mod platform {
    use super::Record;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::fileapi::{FindClose, FindFirstFileW, FindNextFileW};
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::minwinbase::WIN32_FIND_DATAW;
    use winapi::um::winnt::HANDLE;

    pub struct Search {
        handle: HANDLE,
        first: Option<WIN32_FIND_DATAW>,
        exclude: DWORD,
    }
    impl Search {
        pub fn open(
            mask: &[u16],
            _directory: &[u16],
            _pattern: &[u16],
            attrs: i32,
        ) -> Result<Self, i32> {
            let mask: Vec<u16> = mask.iter().copied().chain(Some(0)).collect();
            unsafe {
                let mut data: WIN32_FIND_DATAW = std::mem::zeroed();
                let handle = FindFirstFileW(mask.as_ptr(), &mut data);
                if handle == INVALID_HANDLE_VALUE {
                    return Err(-2);
                }
                Ok(Self {
                    handle,
                    first: Some(data),
                    exclude: (!attrs as u32) & (2 | 4 | 16),
                })
            }
        }
        pub fn next(&mut self) -> Result<Option<Record>, i32> {
            loop {
                let data = if let Some(first) = self.first.take() {
                    first
                } else {
                    unsafe {
                        let mut data: WIN32_FIND_DATAW = std::mem::zeroed();
                        if FindNextFileW(self.handle, &mut data) == 0 {
                            return if GetLastError() == 18 {
                                Ok(None)
                            } else {
                                Err(-2)
                            };
                        }
                        data
                    }
                };
                if data.dwFileAttributes & self.exclude != 0 {
                    continue;
                }
                let len = data
                    .cFileName
                    .iter()
                    .position(|unit| *unit == 0)
                    .unwrap_or(data.cFileName.len());
                let time = (u64::from(data.ftLastWriteTime.dwHighDateTime) << 32)
                    | u64::from(data.ftLastWriteTime.dwLowDateTime);
                return Ok(Some(Record {
                    name: data.cFileName[..len].to_vec(),
                    mtime: time as i64,
                    attrs: data.dwFileAttributes,
                }));
            }
        }
    }
    impl Drop for Search {
        fn drop(&mut self) {
            unsafe {
                FindClose(self.handle);
            }
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::Record;
    use std::ffi::{CStr, CString};

    pub struct Search {
        handle: *mut libc::DIR,
        directory: String,
        pattern: CString,
        exclude: u32,
    }
    impl Search {
        pub fn open(
            _mask: &[u16],
            directory: &[u16],
            pattern: &[u16],
            attrs: i32,
        ) -> Result<Self, i32> {
            let directory = String::from_utf16_lossy(directory);
            // Delphi marshaling to UTF-8 uses NUL-terminated paths/patterns.
            let cstring =
                |s: &str| CString::new(s.split('\0').next().unwrap_or("")).map_err(|_| -1);
            let path = cstring(&directory)?;
            let pattern = cstring(&String::from_utf16_lossy(pattern))?;
            let handle = unsafe { libc::opendir(path.as_ptr()) };
            if handle.is_null() {
                return Err(-2);
            }
            Ok(Self {
                handle,
                directory,
                pattern,
                exclude: (!attrs as u32) & (2 | 4 | 16),
            })
        }
        pub fn next(&mut self) -> Result<Option<Record>, i32> {
            loop {
                unsafe {
                    *libc::__errno_location() = 0;
                    let entry = libc::readdir(self.handle);
                    if entry.is_null() {
                        return if *libc::__errno_location() == 0 {
                            Ok(None)
                        } else {
                            Err(-2)
                        };
                    }
                    let raw = CStr::from_ptr((*entry).d_name.as_ptr());
                    if libc::fnmatch(self.pattern.as_ptr(), raw.as_ptr(), 0) != 0 {
                        continue;
                    }
                    let name = String::from_utf8_lossy(raw.to_bytes());
                    let path =
                        CString::new(format!("{}{}", self.directory, name)).map_err(|_| -1)?;
                    let mut stat: libc::stat = std::mem::zeroed();
                    if libc::lstat(path.as_ptr(), &mut stat) != 0 {
                        continue;
                    }
                    let mut attrs = 0;
                    let kind = stat.st_mode & libc::S_IFMT;
                    if kind == libc::S_IFDIR {
                        attrs |= 16;
                    } else if kind == libc::S_IFLNK {
                        attrs |= 0x400;
                        let mut target: libc::stat = std::mem::zeroed();
                        if libc::stat(path.as_ptr(), &mut target) == 0
                            && target.st_mode & libc::S_IFMT == libc::S_IFDIR
                        {
                            attrs |= 16;
                        }
                    }
                    if libc::euidaccess(path.as_ptr(), libc::W_OK) != 0 {
                        attrs |= 1;
                    }
                    let wide: Vec<u16> = name.encode_utf16().collect();
                    // Exact Delphi RTL behavior: .a and .ab are not faHidden.
                    if wide.len() > 3 && wide.first() == Some(&46) {
                        attrs |= 2;
                    }
                    if attrs & self.exclude != 0 {
                        continue;
                    }
                    return Ok(Some(Record {
                        name: wide,
                        mtime: stat.st_mtime,
                        attrs,
                    }));
                }
            }
        }
    }
    impl Drop for Search {
        fn drop(&mut self) {
            unsafe {
                libc::closedir(self.handle);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SERIAL: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn streaming_queries_preserve_records_and_utf16_names() {
        let root = std::env::temp_dir().join(format!(
            "du-find-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("á😀.txt"), b"fixture").unwrap();
        fs::write(root.join(".hidden"), b"fixture").unwrap();
        fs::create_dir(root.join("child")).unwrap();
        let directory = format!("{}{}", root.display(), std::path::MAIN_SEPARATOR);
        let mask: Vec<u16> = format!("{}*.txt", directory).encode_utf16().collect();
        let dir: Vec<u16> = directory.encode_utf16().collect();
        let pattern: Vec<u16> = "*.txt".encode_utf16().collect();
        unsafe {
            let mut owner = ptr::null_mut();
            assert_eq!(
                du_find_open_legacy(
                    mask.as_ptr(),
                    mask.len(),
                    dir.as_ptr(),
                    dir.len(),
                    pattern.as_ptr(),
                    pattern.len(),
                    63,
                    &mut owner
                ),
                0
            );
            let mut required = 0;
            let mut mtime = 0;
            let mut attrs = 0;
            assert_eq!(
                du_find_next_legacy(
                    owner,
                    ptr::null_mut(),
                    0,
                    &mut required,
                    &mut mtime,
                    &mut attrs
                ),
                0
            );
            assert_eq!(required, "á😀.txt".encode_utf16().count());
            let mut buffer = vec![0u16; required];
            assert_eq!(
                du_find_next_legacy(
                    owner,
                    buffer.as_mut_ptr(),
                    required - 1,
                    &mut required,
                    &mut mtime,
                    &mut attrs
                ),
                -1
            );
            assert_eq!(
                du_find_next_legacy(
                    owner,
                    buffer.as_mut_ptr(),
                    buffer.len(),
                    &mut required,
                    &mut mtime,
                    &mut attrs
                ),
                0
            );
            assert_eq!(String::from_utf16(&buffer).unwrap(), "á😀.txt");
            assert_eq!(
                du_find_next_legacy(
                    owner,
                    ptr::null_mut(),
                    0,
                    &mut required,
                    &mut mtime,
                    &mut attrs
                ),
                1
            );
            du_find_close_legacy(owner);
            let empty: Vec<u16> = "never-matches".encode_utf16().collect();
            let empty_mask: Vec<u16> = format!("{}never-matches", directory)
                .encode_utf16()
                .collect();
            assert_eq!(
                du_find_open_legacy(
                    empty_mask.as_ptr(),
                    empty_mask.len(),
                    dir.as_ptr(),
                    dir.len(),
                    empty.as_ptr(),
                    empty.len(),
                    63,
                    &mut owner
                ),
                -2
            );
            assert!(owner.is_null());
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_rtl_attributes_hidden_lengths_and_directory_symlinks() {
        use std::os::unix::fs::symlink;
        let root = std::env::temp_dir().join(format!(
            "du-find-attrs-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        for name in [".a", ".ab", ".abc", "regular"] {
            fs::write(root.join(name), b"fixture").unwrap();
        }
        fs::create_dir(root.join("directory")).unwrap();
        symlink(root.join("directory"), root.join("link")).unwrap();
        let directory = format!("{}/", root.display());
        let mask: Vec<u16> = format!("{}*", directory).encode_utf16().collect();
        let dir: Vec<u16> = directory.encode_utf16().collect();
        let pattern: Vec<u16> = "*".encode_utf16().collect();
        let mut search = platform::Search::open(&mask, &dir, &pattern, 0).unwrap();
        let mut files = Vec::new();
        while let Some(record) = search.next().unwrap() {
            files.push(String::from_utf16(&record.name).unwrap());
        }
        assert!(files.contains(&".a".to_owned()));
        assert!(files.contains(&".ab".to_owned()));
        assert!(!files.contains(&".abc".to_owned()));
        assert!(!files.contains(&"directory".to_owned()));
        assert!(!files.contains(&"link".to_owned()));
        let mut search = platform::Search::open(&mask, &dir, &pattern, 511).unwrap();
        let mut link_attrs = None;
        while let Some(record) = search.next().unwrap() {
            if String::from_utf16(&record.name).unwrap() == "link" {
                link_attrs = Some(record.attrs);
            }
        }
        assert_eq!(link_attrs.unwrap() & (16 | 0x400), 16 | 0x400);
        drop(search);
        fs::remove_dir_all(root).unwrap();
    }
}
