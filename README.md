# libduallutils

Rust library which provides utility features for Duall ERP.

## C interface

Input strings must be valid UTF-8, NUL-terminated, and backed by readable memory.
Output buffers must have room for the entire result including its NUL terminator.
Invalid UTF-8 and insufficient capacity return `-1`; undersized nonempty buffers
receive an empty string. OS error messages of `du_shutdown`, `du_reboot` and
`du_logout` are the exception: they keep returning `-2` and are truncated on a
UTF-8 character boundary, always NUL-terminated.

`du_lockkey_set` and `du_lockkey_state` have no error result: without a keyboard
(Linux without an X11 display) or on OS failure, `set` does nothing and `state`
returns `false`. The packaging script refuses dirty trees unless `--allow-dirty`
is passed (never for releases). Export names and Pascal binding signatures are unchanged.

`du_execute` requires non-null output, error and exit-code pointers. Its strings
are allocated only after both outputs and the exit status have been validated.
Non-UTF-8 or embedded NUL in process output returns `-3`, with null strings and
exit code `-1`. Valid UTF-8 is preserved. Programs producing other encodings must
be configured for UTF-8; the interface does not guess encodings. Free successful
output strings with `du_dispose`.

`du_spawn` requires a non-null exit-code pointer only when `waiting` is true. When
a pointer is supplied it is initialized to `-1` before the child starts, including
the non-waiting case. A child terminated by a signal (`waiting` true) returns `-3`
and the exit code stays `-1`.

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

## Linux SONAME and binary packages

Linux builds embed `libduallutils.so.4` as their SONAME. The binary package contains
`libduallutils.so.4.0.3` and links `libduallutils.so.4` and `libduallutils.so`,
preserving C linker and Pascal loader names. Windows packages contain `duallutils.dll`.

Build and package x64 artifacts (Python 3.11+):

```text
cargo build --release --target x86_64-unknown-linux-gnu
python3 scripts/package_release.py --target x86_64-unknown-linux-gnu
```

On Windows, use `x86_64-pc-windows-msvc` and set
`RUSTFLAGS=-Ctarget-feature=+crt-static` before building. Packages in `dist/`
include bindings, license, the committed Cargo.lock, BUILD.json and SHA256SUMS;
each archive has its own `.sha256`. BUILD.json records source revision, dirty
state, toolchain and, on Linux, SONAME, the minimum required glibc (strong symbol versions only) and the
optional newer glibc versions the binary uses when available (weak, with fallback). Linux also
requires the X11 runtime (`libX11.so.6`); keyboard helpers require a working X11
session. Build machines need the X11 development package. Newer distributions
may produce binaries requiring newer glibc on deployment machines.
