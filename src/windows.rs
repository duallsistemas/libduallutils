use libc::c_int;
use std::io::{Error, ErrorKind};
use std::mem;
use winapi::shared::minwindef::{DWORD, WORD};
use winapi::um::minwinbase::SYSTEMTIME;
use winapi::um::sysinfoapi::SetSystemTime;

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
    if (SetSystemTime(&st) == 0) && (Error::last_os_error().kind() == ErrorKind::PermissionDenied) {
        return -2;
    }
    0
}
