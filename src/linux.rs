use libc::{c_int, mktime, settimeofday, time_t, timeval, EPERM};
use std::io::Error;
use std::mem;
use std::ptr;

pub unsafe fn datetime_set(
    year: c_int,
    month: c_int,
    day: c_int,
    hour: c_int,
    minute: c_int,
    second: c_int,
) -> c_int {
    let mut time: libc::tm = mem::zeroed();
    time.tm_year = year - 1900;
    time.tm_mon = month - 1;
    time.tm_mday = day;
    time.tm_hour = hour;
    time.tm_min = minute;
    time.tm_sec = second;
    if time.tm_year < 0 {
        time.tm_year = 0;
    }
    let t: time_t = mktime(&mut time);
    if t == -1 {
        return -1;
    }
    let tv: timeval = timeval {
        tv_sec: t,
        tv_usec: 0,
    };
    if settimeofday(&tv, ptr::null()) == -1 {
        if Error::last_os_error().raw_os_error().unwrap() == EPERM {
            return -2;
        } else {
            return -3;
        }
    };
    0
}
