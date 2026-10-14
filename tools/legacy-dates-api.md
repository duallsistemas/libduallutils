# Legacy dates and read-only wait API

These exports accept UTF-16 units and explicit lengths. Embedded NUL ends the
path. Input pointers must be valid and aligned; null with zero length is
allowed. Non-null output pointers must be writable and non-overlapping.

`du_file_age_legacy(path, length, raw)` returns 0 on success, -1 invalid input,
-2 OS/conversion failure, -3 caught panic. On failure `raw` is untouched.
Windows returns a local FILETIME as 64 raw bits after FileTimeToLocalFileTime,
using the current timezone offset, as Delphi FileAgeInternal does. It rejects
directories; a sharing/lock failure falls back to FindFirstFile metadata. A
reparse target is resolved only if its GENERIC_READ/FILE_SHARE_READ handle can
be opened, using GetFinalPathNameByHandleW. Linux stat follows symbolic links,
accepts directories and returns st_mtime seconds. Delphi handles TDateTime
conversion and its platform-specific integer narrowing.

`du_set_file_date_legacy(path, length, packed_date)` uses the same status codes.
On Windows packed_date carries the DOS date/time integer in its low32 bits: CreateFileW uses GENERIC_WRITE,
share zero and OPEN_EXISTING; DOS local time converts to UTC and only last-write
time changes. On Linux the signed i64 seconds are supplied to both atime and
mtime via utime. Delphi performs DateTimeToFileDate before invoking the export,
including validation exceptions, DOS two-second rounding and platform LongInt
range semantics (32 bits on Win64, 64 bits in this Delphi Linux64 consumer). The existing SetFileDateTime facade ignores OS failure status.

`du_wait_writable_legacy(path, length, result)` returns 0 with a byte result:
1 if the read-only check stops, 0 after 101 iterations with 100ms sleeps each.
It blocks the calling thread for up to ~10.1s with no cancellation; do not call
it from a UI thread without moving it to a worker.
There is no existence guard in Rust; the dUtils facade retains its missing-file
return true. The Windows probe reads FILE_ATTRIBUTE_READONLY and follows a
resolvable reparse target. A missing initial path reports not read-only. Linux
uses euidaccess(R_OK)==0 and euidaccess(W_OK)!=0. This is the IsFileInUse2 loop,
not an exclusive-open probe and not a proof that a later write will succeed.
Errors -1/-3 preserve the result byte. Unit tests inject probes and sleepers to
verify 101 iterations and transitions without imposing a real 10.1s delay.

Optional Pascal bindings expose dUtils.LegacyFileAge, LegacySetFileDate and
LegacyWaitWritable. LegacyFileAge returns false for -2 while leaving the scalar
output unchanged. FileDateTime's original unchecked failure output remains
undefined; these exports do not initialize it to a new sentinel.

Behavior was independently derived from the installed Delphi 13 RTL 37.0
System.SysUtils.pas FileAgeInternal, FileSetDate and FileIsReadOnly. Windows
symbolic-link fixture creation was unavailable on this machine (OS error1314);
Linux link behavior is validated separately.
