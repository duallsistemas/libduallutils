use libc::c_int;
use std::io::Error;
use std::mem;
use winapi::shared::minwindef::{DWORD, WORD};
use winapi::shared::winerror::ERROR_PRIVILEGE_NOT_HELD;
use winapi::um::minwinbase::SYSTEMTIME;
use winapi::um::sysinfoapi::SetLocalTime;

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
