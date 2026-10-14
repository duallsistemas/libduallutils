use crypto::digest::Digest;
use crypto::md5::Md5;
use std::fs::OpenOptions;
use std::io::{ErrorKind, Read};

pub fn md5_file(filename: &str) -> Result<String, i32> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use winapi::um::winnt::FILE_SHARE_READ;
        // Delphi fmOpenRead | fmShareDenyWrite permits readers, denies writers/deletion.
        options.share_mode(FILE_SHARE_READ);
    }
    let mut file = options.open(filename).map_err(|error| if error.kind() == ErrorKind::NotFound { -2 } else { -3 })?;
    let mut hasher = Md5::new();
    let mut buffer = [0u8; 16384];
    loop {
        match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => hasher.input(&buffer[..count]),
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(_) => return Err(-3),
        }
    }
    Ok(hasher.result_str())
}

// Delphi Trim removes characters <= #32; Unicode whitespace is not equivalent.
fn trim(value: &str) -> &str { value.trim_matches(|c| c <= ' ') }

#[cfg(windows)]
pub fn run_command(command: &str, params: &str, waiting: bool, window_state: u16) -> i32 {
    use std::ptr;
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::{CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW};
    use winapi::um::shellapi::ShellExecuteW;
    use winapi::um::synchapi::WaitForSingleObject;
    use winapi::um::winbase::{CREATE_NEW_CONSOLE, INFINITE, NORMAL_PRIORITY_CLASS, STARTF_USESHOWWINDOW, WAIT_OBJECT_0};
    let command = trim(command);
    let params = trim(params);
    let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
    unsafe {
        if !waiting {
            let verb = wide("open");
            let command = wide(command);
            let params = wide(params);
            let result = ShellExecuteW(ptr::null_mut(), verb.as_ptr(), command.as_ptr(), params.as_ptr(), ptr::null(), i32::from(window_state)) as isize;
            return if result > 32 { 0 } else if result == 2 || result == 3 { -2 } else { -3 };
        }
        let mut full = wide(&format!("{} {}", command, params));
        let mut startup: STARTUPINFOW = std::mem::zeroed();
        startup.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        startup.dwFlags = STARTF_USESHOWWINDOW;
        startup.wShowWindow = window_state;
        let mut process: PROCESS_INFORMATION = std::mem::zeroed();
        if CreateProcessW(ptr::null(), full.as_mut_ptr(), ptr::null_mut(), ptr::null_mut(), 0,
            CREATE_NEW_CONSOLE | NORMAL_PRIORITY_CLASS, ptr::null_mut(), ptr::null(), &mut startup, &mut process) == 0 {
            return match GetLastError() { 2 | 3 => -2, _ => -3 };
        }
        let result = WaitForSingleObject(process.hProcess, INFINITE);
        CloseHandle(process.hThread);
        CloseHandle(process.hProcess);
        if result == WAIT_OBJECT_0 { 0 } else { -3 }
    }
}

#[cfg(target_os = "linux")]
pub fn run_command(command: &str, params: &str, waiting: bool, _window_state: u16) -> i32 {
    // Keep system() shell expansion, redirects and caller-supplied quotes.
    let full = format!("{} {}{}", trim(command), trim(params), if waiting { "" } else { " &" });
    match std::process::Command::new("/bin/sh").arg("-c").arg(full).status() {
        Ok(_) => 0,
        Err(error) if error.kind() == ErrorKind::NotFound => -2,
        Err(_) => -3,
    }
}
