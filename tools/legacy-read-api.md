# Borrowed handles and byte ownership

`du_read_file_legacy(path, len, data, length, owner, os_error)` opens a native
read handle, captures its initial metadata size and reads exactly that size.
Windows uses UTF-16 paths and FILE_SHARE_READ; Linux uses UTF-8 conversion and
O_RDONLY. Embedded NUL ends either path. Empty files still return an owner.

`du_read_handle_legacy(handle, initial_size, data, length, owner, os_error)`
borrows a handle opened by Delphi, reads at its current position and never
closes it. This keeps validation, file creation and RTL exception contracts in
the facade. Size is u64; lengths and handles are pointer-sized unsigned values.
Growth beyond the initial size is ignored; insufficient bytes returns -3.

`du_write_handle_legacy(handle, data, length, os_error)` writes all bytes at the
current position and never closes the borrowed handle. Creation, seeks and
truncation remain with the caller.

Read success returns 0 with immutable bytes valid until
`du_read_file_dispose(owner)`. The caller must copy before disposal and dispose
exactly once, even when length is zero. Dispose accepts null. Output pointers
must be valid, aligned and non-overlapping. Failure clears data/length/owner
and reports a native OS error when available. Status -1 means invalid argument,
-2 open/initial metadata failure, -3 read/write failure, -4 caught panic or
allocation failure. End-of-file errors can report OS code zero.

Bindings expose optional functions and `dUtils.LegacyReadFile`,
`LegacyReadHandle`, `LegacyWriteHandle`, `LegacyFindOpen`, `LegacyFindNext` and
`LegacyFindClose`. Read wrappers copy to TBytes and dispose in finally. Text
BOM/ANSI/UTF-8 decoding and encoding stay in Delphi.
