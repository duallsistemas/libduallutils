use libc::{c_char, c_int};
use std::ffi::{CStr, OsString};
use std::io::Error;
use std::mem;
use std::os::windows::prelude::*;
use winapi::shared::minwindef::{DWORD, MAX_PATH, WORD};
use winapi::shared::winerror::{ERROR_ACCESS_DENIED, ERROR_PRIVILEGE_NOT_HELD};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::minwinbase::SYSTEMTIME;
use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
use winapi::um::sysinfoapi::SetLocalTime;
use winapi::um::winnt::PROCESS_TERMINATE;
use winapi::um::{
    handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
    tlhelp32::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    },
};

pub unsafe fn datetime_set(
    year: c_int,
    month: c_int,
    day: c_int,
    hour: c_int,
    minute: c_int,
    second: c_int,
) -> c_int {
    let mut st: SYSTEMTIME = mem::zeroed();
    st.wYear = year as WORD;
    st.wMonth = month as WORD;
    st.wDay = day as WORD;
    st.wHour = hour as WORD;
    st.wMinute = minute as WORD;
    st.wSecond = second as WORD;
    if SetLocalTime(&st) == 0 {
        if Error::last_os_error().raw_os_error().unwrap() as DWORD == ERROR_PRIVILEGE_NOT_HELD {
            return -2;
        } else {
            return -3;
        }
    }
    0
}

pub unsafe fn terminate(process_name: *const c_char) -> c_int {
    let handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if handle == INVALID_HANDLE_VALUE {
        return -4;
    }
    let mut process_entry: PROCESSENTRY32W = PROCESSENTRY32W {
        dwSize: mem::size_of::<PROCESSENTRY32W>() as u32,
        cntUsage: 0,
        th32ProcessID: 0,
        th32DefaultHeapID: 0,
        th32ModuleID: 0,
        cntThreads: 0,
        th32ParentProcessID: 0,
        pcPriClassBase: 0,
        dwFlags: 0,
        szExeFile: [0; MAX_PATH],
    };
    let ret: c_int = {
        match Process32FirstW(handle, &mut process_entry) {
            1 => loop {
                let name: String = process_entry
                    .szExeFile
                    .iter()
                    .map(|&v| (v & 0xFF) as u8)
                    .take_while(|&c| c != 0)
                    .map(|c| c as char)
                    .collect();
                if name.to_ascii_lowercase()
                    == CStr::from_ptr(process_name)
                        .to_string_lossy()
                        .to_ascii_lowercase()
                {
                    let process = OpenProcess(PROCESS_TERMINATE, 0, process_entry.th32ProcessID);
                    if process != std::ptr::null_mut() {
                        if TerminateProcess(process, 0) > 0 {
                            CloseHandle(process);
                            return 0;
                        }
                        if GetLastError() == ERROR_ACCESS_DENIED {
                            return -3;
                        }
                    } else {
                        return -3;
                    }
                }
                if Process32NextW(handle, &mut process_entry) != 1 {
                    return -2;
                }
            },
            0 => -2,
            _ => -4,
        };
        -4
    };
    CloseHandle(handle);
    ret
}
