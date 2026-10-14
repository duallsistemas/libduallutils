use super::*;
use std::ffi::CString;
use std::io::Write;

#[test]
fn hash_buffers_reject_truncation_and_preserve_canary() {
    let input = CString::new("abc123").unwrap();
    let mut buffer = [b'X' as c_char; 64];
    unsafe {
        assert_eq!(du_md5(input.as_ptr(), buffer.as_mut_ptr(), 4), -1);
        assert_eq!(buffer[0], 0);
        assert_eq!(buffer[4], b'X' as c_char);
        assert_eq!(du_sha1(input.as_ptr(), buffer.as_mut_ptr(), 1), -1);
        assert_eq!(buffer[0], 0);
        assert_eq!(du_md5(input.as_ptr(), buffer.as_mut_ptr(), 33), 0);
        assert_eq!(from_c_str!(buffer.as_ptr()).unwrap(), "e99a18c428cb38d5f260853678922e03");
        assert_eq!(buffer[32], 0);
        assert_eq!(buffer[33], b'X' as c_char);
    }
}

#[test]
fn invalid_utf8_and_glob_return_argument_errors() {
    let input = [0xffu8, 0];
    let input = input.as_ptr() as *const c_char;
    let mut buffer = [0 as c_char; 64];
    let pattern = CString::new("[").unwrap();
    unsafe {
        assert_eq!(du_md5(input, buffer.as_mut_ptr(), buffer.len()), -1);
        assert_eq!(du_sha1(input, buffer.as_mut_ptr(), buffer.len()), -1);
        assert_eq!(du_md5_file(input, buffer.as_mut_ptr(), buffer.len()), -1);
        assert_eq!(du_sha1_file(input, buffer.as_mut_ptr(), buffer.len()), -1);
        assert_eq!(du_open(input), -1);
        assert_eq!(du_once(input), -1);
        assert_eq!(du_terminate(input), -1);
        assert_eq!(du_deltree(input), -1);
        assert_eq!(du_deltree(pattern.as_ptr()), -1);
    }
}

#[test]
fn process_requires_output_pointers() {
    let program = CString::new("unused-program").unwrap();
    let mut code = 0;
    unsafe {
        assert_eq!(du_spawn(program.as_ptr(), ptr::null(), ptr::null(), ptr::null(), true, ptr::null_mut()), -1);
        assert_eq!(du_execute(program.as_ptr(), ptr::null(), ptr::null(), ptr::null(), ptr::null_mut(), ptr::null_mut(), &mut code), -1);
    }
}

// Este teste vira processo filho apenas quando a fixture e pedida explicitamente.
#[test]
fn output_fixture() {
    if let Ok(kind) = std::env::var("DUALL_OUTPUT_FIXTURE") {
        match kind.as_str() {
            "utf8" => std::io::stdout().write_all(&[0xc3, 0xa7]).unwrap(),
            "invalid" => std::io::stdout().write_all(&[0xff]).unwrap(),
            "stderr" => std::io::stderr().write_all(&[0xff]).unwrap(),
            "nul" => std::io::stdout().write_all(&[65, 0, 66]).unwrap(),
            _ => panic!("unexpected fixture"),
        }
    }
}

#[test]
fn process_outputs_are_validated_before_ownership_transfer() {
    let program = CString::new(std::env::current_exe().unwrap().to_str().unwrap()).unwrap();
    let args_owned: Vec<CString> = ["--exact", "regression::output_fixture", "--nocapture"]
        .iter().map(|value| CString::new(*value).unwrap()).collect();
    let mut args: Vec<*const c_char> = args_owned.iter().map(|value| value.as_ptr()).collect();
    args.push(ptr::null());
    for kind in ["invalid", "stderr", "nul", "utf8"].iter() {
        let env = CString::new(format!("DUALL_OUTPUT_FIXTURE={}", kind)).unwrap();
        let envs = [env.as_ptr(), ptr::null()];
        let mut stdout = ptr::null_mut();
        let mut stderr = ptr::null_mut();
        let mut code = 123;
        unsafe {
            let result = du_execute(program.as_ptr(), ptr::null(), args.as_ptr(), envs.as_ptr(), &mut stdout, &mut stderr, &mut code);
            if *kind == "utf8" {
                assert_eq!(result, 0);
                assert!(from_c_str!(stdout).unwrap().contains('\u{e7}'));
                assert_eq!(code, 0);
                du_dispose(stdout);
                du_dispose(stderr);
            } else {
                assert_eq!(result, -3);
                assert!(stdout.is_null());
                assert!(stderr.is_null());
                assert_eq!(code, -1);
            }
        }
    }
}

#[cfg(windows)]
#[test]
fn deletion_reports_sharing_violation() {
    use std::os::windows::fs::OpenOptionsExt;
    let path = std::env::temp_dir().join(format!("duall-delete-{}-{}.txt", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let locked = fs::OpenOptions::new().write(true).create_new(true).share_mode(0).open(&path).unwrap();
    let name = CString::new(path.to_str().unwrap()).unwrap();
    let result = unsafe { du_deltree(name.as_ptr()) };
    let exists = path.exists();
    drop(locked);
    fs::remove_file(&path).unwrap();
    assert_eq!(result, -3);
    assert!(exists);
    assert_eq!(unsafe { du_deltree(name.as_ptr()) }, -2);
}

#[test]
fn os_error_messages_are_truncated_with_nul_on_char_boundary() {
    let mut buffer = [b'X' as c_char; 8];
    unsafe {
        utils::copy_error_msg("a\u{e7}\u{e7}\u{e7}", buffer.as_mut_ptr(), 5);
        assert_eq!(from_c_str!(buffer.as_ptr()).unwrap(), "a\u{e7}");
        assert_eq!(buffer[5], b'X' as c_char);
        utils::copy_error_msg("abc", buffer.as_mut_ptr(), 1);
        assert_eq!(buffer[0], 0);
        utils::copy_error_msg("abc", buffer.as_mut_ptr(), 8);
        assert_eq!(from_c_str!(buffer.as_ptr()).unwrap(), "abc");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn lock_keys_without_display_do_not_crash() {
    if std::env::var_os("DISPLAY").is_some() {
        return;
    }
    unsafe {
        du_lockkey_set(DU_LOCKKEY::DU_LK_CAPSLOCK, true);
        assert!(!du_lockkey_state(DU_LOCKKEY::DU_LK_CAPSLOCK));
    }
}
