# Legacy file enumeration ABI

These C ABI exports support the Delphi dUtils adapter. All strings use UTF-16
code units and explicit lengths. Lengths and capacities count code units, not
bytes. Non-null pointers must refer to valid, aligned storage.

```c
int32_t du_find_open_legacy(
    const uint16_t *mask, size_t masklen,
    const uint16_t *directory, size_t dirlen,
    const uint16_t *pattern, size_t patlen,
    int32_t attrs, void **out_owner);

int32_t du_find_next_legacy(
    void *owner, uint16_t *name, size_t capacity,
    size_t *required, int64_t *mtime, uint32_t *fileattrs);

void du_find_close_legacy(void *owner);
```

## Ownership and streaming

Open returns an opaque owner with the first matching record pending. It sets
`out_owner` to NULL on failure. Close every successful owner exactly once,
including when a Delphi allocation or subsequent operation raises an exception.
Owners cannot be shared concurrently. Close accepts NULL.

Next with `name = NULL` and `capacity = 0` queries the pending record. It writes
the required UTF-16 length, modification time and attributes without consuming
the record. An insufficient buffer also retains the pending record. A successful
copy writes exactly `required` code units without a terminating NUL and consumes
the record. The following next call fetches another native record.

`required`, `mtime` and `fileattrs` must be non-null. EOF leaves the previous
Delphi search record unchanged because the adapter copies only on success.

## Status values

| Status | Open | Next |
| --- | --- | --- |
| 0 | Owner created, first record pending | Record available or copied |
| 1 | Unused | End of enumeration |
| -1 | Invalid argument | Invalid argument or insufficient buffer |
| -2 | OS failure or no matching entry | OS failure |
| -3 | Caught Rust panic | Caught Rust panic |

These statuses are library codes, not Win32 error codes or errno values.
Invalid or already-closed non-null owners violate the caller contract.

## Platform semantics

Windows uses the full mask with FindFirstFileW and FindNextFileW. Directory and
pattern are ignored after pointer validation. Native enumeration order and
wildcard behavior are preserved. Attributes are the native WIN32_FIND_DATAW
flags. Modification time is the raw 64-bit UTC FILETIME bit pattern; the Delphi
adapter assigns it to TSearchRec.FindData.ftLastWriteTime so the RTL TimeStamp
property retains the original local-time conversion.

Linux uses opendir/readdir in native order and fnmatch with flags 0, preserving
the case-sensitive filename masks used by the Delphi Linux RTL. The caller
supplies the directory with a trailing separator and the filename pattern;
when ExtractFilePath is empty, the Delphi adapter supplies the current directory.
Paths and patterns are converted from UTF-16 to UTF-8, replacing malformed
UTF-16 and stopping at embedded NUL.

Linux metadata comes from lstat. Modification time is st_mtime in seconds.
Directory is 0x10; symlink is 0x400, with directory also set for links targeting a
directory. Read-only is determined using euidaccess with W_OK. The Delphi RTL's
hidden-name policy is preserved: leading dot names are hidden only when their
UTF-16 length exceeds three code units.

Both platforms exclude entries matching `(~attrs) & (0x02 | 0x04 | 0x10)`,
as Delphi FindFirst does. Linux entries whose lstat fails are skipped rather
than exposing the RTL's undefined metadata after a concurrent deletion.

List ownership, recursion, extension matching, sorting, date formatting and
deletion policy remain in Delphi; no Delphi object crosses this ABI.
