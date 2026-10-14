# libduallutils

Rust library which provides utility features for Duall ERP.

## C interface

Input strings must be valid UTF-8, NUL-terminated, and backed by readable memory.
Output buffers must have room for the entire result including its NUL terminator.
Invalid UTF-8 and insufficient capacity return `-1`; undersized nonempty buffers
receive an empty string. Export names and Pascal binding signatures are unchanged.

`du_execute` requires non-null output, error and exit-code pointers. Its strings
are allocated only after both outputs and the exit status have been validated.
Non-UTF-8 or embedded NUL in process output returns `-3`, with null strings and
exit code `-1`. Valid UTF-8 is preserved. Programs producing other encodings must
be configured for UTF-8; the interface does not guess encodings. Free successful
output strings with `du_dispose`.

`du_deltree` returns `-1` for an invalid pattern, `-2` for no matches, and `-3`
for traversal or removal failures. Earlier matches may already have been removed
before a later failure; deletion is not transactional. `sysinfo` is a Linux-only
dependency because the Windows implementation uses Win32 APIs directly.

Focused regression tests use local fixtures and child processes:

```powershell
cargo test --target x86_64-pc-windows-msvc regression:: -- --test-threads=1
```

This filter excludes older tests that depend on platform executables or alter
keyboard lock-key state.
