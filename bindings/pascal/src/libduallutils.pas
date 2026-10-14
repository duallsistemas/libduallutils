unit libduallutils;

{$IFDEF FPC}
 {$MODE DELPHI}
 {$PACKRECORDS C}
 {$IFDEF VER3_0}
  {$PUSH}{$MACRO ON}
  {$DEFINE MarshaledAString := PAnsiChar}
  {$DEFINE PMarshaledAString := PPAnsiChar}
  {$DEFINE EInvalidOpException := Exception}
  {$IFDEF VER3_0_0}
   {$DEFINE EFileNotFoundException := Exception}
  {$ENDIF}
  {$POP}
 {$ENDIF}
{$ENDIF}

interface

uses
  SysUtils,
  StrUtils,
{$IFDEF MSWINDOWS}
  Windows,
{$ENDIF}
{$IFDEF FPC}
  DynLibs,
{$ENDIF}
  SyncObjs;


const
  SharedPrefix = {$IFDEF MSWINDOWS}''{$ELSE}'lib'{$ENDIF};
{$IF (NOT DEFINED(FPC)) OR DEFINED(VER3_0)}
  SharedSuffix =
 {$IF DEFINED(MSWINDOWS)}
    'dll'
 {$ELSEIF DEFINED(MACOS)}
    'dylib'
 {$ELSE}
    'so'
 {$ENDIF};
{$ENDIF}
  DU_LIB_NAME = Concat(SharedPrefix, 'duallutils.', SharedSuffix);

{$IFDEF FPC}
 {$IFDEF VER3_0}
const
  NilHandle = DynLibs.NilHandle;
type
  TLibHandle = DynLibs.TLibHandle;
 {$ENDIF}
{$ELSE}
const
  NilHandle = HMODULE(0);
type
  TLibHandle = HMODULE;
{$ENDIF}

const
  DU_ERR_SIZE = 256;

resourcestring
  SduLibEmptyName = 'Empty library name.';
  SduLibNotLoaded = 'Library ''%s'' not loaded.';
  SduLibInvalid = 'Invalid library ''%s''.';

type
  Pcvoid = Pointer;
  Pcchar = MarshaledAString;
  PPcchar = PMarshaledAString;
  cchar = Byte;
  cbool = Boolean;
  cint = Integer;
  Pcint= PInteger;
  csize_t = NativeUInt;
  cenum = cint;

  TLegacyMathCallback = function(op: Cardinal; a, b: Double; n: Integer;
    input_i64: Int64; output: PDouble; integer_output: PInt64;
    context: Pointer): Integer; cdecl;
  TLegacyFloatParser = function(context: Pointer; input: PWord;
    inputlen: NativeUInt; output: PDouble): Integer; cdecl;

  EduLibNotLoaded = class(EFileNotFoundException);

type
  DU_LOCKKEY = cenum;
const
  DU_LK_CAPSLOCK = 0;
  DU_LK_NUMLOCK = 1;
  DU_LK_SCROLLLOCK = 2;

var
  du_version: function: Pcchar; cdecl;
  du_dispose: procedure(cstr: Pcchar); cdecl;
  du_md5: function(const cstr: Pcchar; md5: Pcchar; size: csize_t): cint; cdecl;
  du_md5_file: function(const filename: Pcchar; md5: Pcchar;
    size: csize_t): cint; cdecl;
  du_md5_file_legacy: function(const filename: Pcchar; md5: Pcchar;
    size: csize_t): cint; cdecl;
  du_run_command_legacy: function(const command, params: Pcchar;
    waiting: cbool; window_state: Word): cint; cdecl;
  du_open_url_legacy: function(const url: Pcchar): cint; cdecl;
  du_legacy_text: function(op: Cardinal; a: PWord; alen: NativeUInt;
    b: PWord; blen: NativeUInt; n: Integer; c: Word; output: PWord;
    capacity: NativeUInt; required: PNativeUInt): Integer; cdecl;
  du_legacy_replace: function(a: PWord; alen: NativeUInt; old: PWord;
    oldlen: NativeUInt; replacement: PWord; replacementlen: NativeUInt;
    output: PWord; capacity: NativeUInt; required: PNativeUInt): Integer; cdecl;
  du_legacy_file: function(op: Cardinal; path: PWord; pathlen: NativeUInt;
    value: PInt64): Integer; cdecl;
  du_read_file_legacy: function(path: PWord; pathlen: NativeUInt;
    out_data: PPointer; out_len: PNativeUInt; out_owner: PPointer;
    out_os_error: PInteger): Integer; cdecl;
  du_read_handle_legacy: function(handle: NativeUInt; size: UInt64;
    out_data: PPointer; out_len: PNativeUInt; out_owner: PPointer;
    out_os_error: PInteger): Integer; cdecl;
  du_write_handle_legacy: function(handle: NativeUInt; data: Pointer; size: NativeUInt;
    out_os_error: PInteger): Integer; cdecl;
  du_read_file_dispose: procedure(owner: Pointer); cdecl;
  du_find_open_legacy: function(mask: PWord; masklen: NativeUInt;
    directory: PWord; dirlen: NativeUInt; pattern: PWord; patlen: NativeUInt;
    attrs: Integer; out_owner: PPointer): Integer; cdecl;
  du_find_next_legacy: function(owner: Pointer; name: PWord; capacity: NativeUInt;
    required: PNativeUInt; mtime: PInt64; fileattrs: PCardinal): Integer; cdecl;
  du_find_close_legacy: procedure(owner: Pointer); cdecl;
  du_file_age_legacy: function(path: PWord; pathlen: NativeUInt;
    out_raw: PInt64): Integer; cdecl;
  du_set_file_date_legacy: function(path: PWord; pathlen: NativeUInt;
    packed_date: Int64): Integer; cdecl;
  du_wait_writable_legacy: function(path: PWord; pathlen: NativeUInt;
    writable: PByte): Integer; cdecl;
  du_math_legacy: function(op: Cardinal; a, b: Double; dec, kind: Integer;
    callback: TLegacyMathCallback; context: Pointer; output: PDouble): Integer; cdecl;
  du_string_to_float_def_legacy: function(input: PWord; inputlen: NativeUInt;
    separator: Word; default_value: Double; parser: TLegacyFloatParser;
    context: Pointer; output: PDouble): Integer; cdecl;
  du_sha1: function(const cstr: Pcchar; sha1: Pcchar; size: csize_t): cint; cdecl;
  du_sha1_file: function(const filename: Pcchar; sha1: Pcchar;
    size: csize_t): cint; cdecl;
  du_spawn: function(const &program: Pcchar; const workdir: Pcchar;
    const args: PPcchar; const envs: PPcchar; waiting: cbool;
    exitcode: Pcint): cint; cdecl;
  du_execute: function(const &program: Pcchar; const workdir: Pcchar;
    const args: PPcchar; const envs: PPcchar; output: PPcchar;
    error: PPcchar; exitcode: Pcint): cint; cdecl;
  du_open: function(const filename: Pcchar): cint; cdecl;
  du_once: function(const ident: Pcchar): cint; cdecl;
  du_shutdown: function(forced: cbool; error_msg: Pcchar;
    error_size: csize_t): cint; cdecl;
  du_reboot: function(forced: cbool; error_msg: Pcchar;
    error_size: csize_t): cint; cdecl;
  du_logout: function(forced: cbool; error_msg: Pcchar;
    error_size: csize_t): cint; cdecl;
  du_datetime_set: function(year: cint; month: cint; day: cint; hour: cint;
    minute: cint; second: cint): cint; cdecl;
  du_lockkey_set: procedure(key: DU_LOCKKEY; enabled: cbool); cdecl;
  du_lockkey_state: function(key: DU_LOCKKEY): cbool; cdecl;
  du_terminate: function(const process_name: Pcchar): cint; cdecl;
  du_deltree: function(const pattern: Pcchar): cint; cdecl;

function TryLoad(const ALibraryName: TFileName): Boolean;

procedure Load(const ALibraryName: TFileName);

procedure Unload;

procedure Check;

implementation

var
  GCS: TCriticalSection;
  GLibHandle: TLibHandle = NilHandle;
  GLibLastName: TFileName = '';

function TryLoad(const ALibraryName: TFileName): Boolean;
begin
  if ALibraryName = '' then
    raise EArgumentException.Create(SduLibEmptyName);
  GCS.Acquire;
  try
    if GLibHandle <> NilHandle then
      FreeLibrary(GLibHandle);
    GLibHandle := SafeLoadLibrary(ALibraryName);
    if GLibHandle = NilHandle then
      Exit(False);
    GLibLastName := ALibraryName;
    du_version := GetProcAddress(GLibHandle, 'du_version');
    du_dispose := GetProcAddress(GLibHandle, 'du_dispose');
    du_md5 := GetProcAddress(GLibHandle, 'du_md5');
    du_md5_file := GetProcAddress(GLibHandle, 'du_md5_file');
    du_md5_file_legacy := GetProcAddress(GLibHandle, 'du_md5_file_legacy');
    du_run_command_legacy := GetProcAddress(GLibHandle, 'du_run_command_legacy');
    du_open_url_legacy := GetProcAddress(GLibHandle, 'du_open_url_legacy');
    du_legacy_text := GetProcAddress(GLibHandle, 'du_legacy_text');
    du_legacy_replace := GetProcAddress(GLibHandle, 'du_legacy_replace');
    du_legacy_file := GetProcAddress(GLibHandle, 'du_legacy_file');
    du_read_file_legacy := GetProcAddress(GLibHandle, 'du_read_file_legacy');
    du_read_handle_legacy := GetProcAddress(GLibHandle, 'du_read_handle_legacy');
    du_write_handle_legacy := GetProcAddress(GLibHandle, 'du_write_handle_legacy');
    du_read_file_dispose := GetProcAddress(GLibHandle, 'du_read_file_dispose');
    du_find_open_legacy := GetProcAddress(GLibHandle, 'du_find_open_legacy');
    du_find_next_legacy := GetProcAddress(GLibHandle, 'du_find_next_legacy');
    du_find_close_legacy := GetProcAddress(GLibHandle, 'du_find_close_legacy');
    du_file_age_legacy := GetProcAddress(GLibHandle, 'du_file_age_legacy');
    du_set_file_date_legacy := GetProcAddress(GLibHandle, 'du_set_file_date_legacy');
    du_wait_writable_legacy := GetProcAddress(GLibHandle, 'du_wait_writable_legacy');
    du_math_legacy := GetProcAddress(GLibHandle, 'du_math_legacy');
    du_string_to_float_def_legacy := GetProcAddress(GLibHandle, 'du_string_to_float_def_legacy');
    du_sha1 := GetProcAddress(GLibHandle, 'du_sha1');
    du_sha1_file := GetProcAddress(GLibHandle, 'du_sha1_file');
    du_spawn := GetProcAddress(GLibHandle, 'du_spawn');
    du_execute := GetProcAddress(GLibHandle, 'du_execute');
    du_open := GetProcAddress(GLibHandle, 'du_open');
    du_once := GetProcAddress(GLibHandle, 'du_once');
    du_shutdown := GetProcAddress(GLibHandle, 'du_shutdown');
    du_reboot := GetProcAddress(GLibHandle, 'du_reboot');
    du_logout := GetProcAddress(GLibHandle, 'du_logout');
    du_lockkey_set := GetProcAddress(GLibHandle, 'du_lockkey_set');
    du_lockkey_state := GetProcAddress(GLibHandle, 'du_lockkey_state');
    du_datetime_set := GetProcAddress(GLibHandle, 'du_datetime_set');
    du_terminate := GetProcAddress(GLibHandle, 'du_terminate');
    du_deltree := GetProcAddress(GLibHandle, 'du_deltree');
    Result := True;
  finally
    GCS.Release;
  end;
end;

procedure Load(const ALibraryName: TFileName);
begin
  if not TryLoad(ALibraryName) then
  begin
{$IFDEF MSWINDOWS}
    if GetLastError = ERROR_BAD_EXE_FORMAT then
      raise EduLibNotLoaded.CreateFmt(SduLibInvalid, [ALibraryName]);
{$ENDIF}
    raise EduLibNotLoaded.CreateFmt(SduLibNotLoaded, [ALibraryName])
  end;
end;

procedure Unload;
begin
  GCS.Acquire;
  try
    if (GLibHandle = NilHandle) or (not FreeLibrary(GLibHandle)) then
      Exit;
    GLibHandle := NilHandle;
    GLibLastName := '';
    du_version := nil;
    du_dispose := nil;
    du_md5 := nil;
    du_md5_file := nil;
    du_md5_file_legacy := nil;
    du_run_command_legacy := nil;
    du_open_url_legacy := nil;
    du_legacy_text := nil;
    du_legacy_replace := nil;
    du_legacy_file := nil;
    du_read_file_legacy := nil;
    du_read_handle_legacy := nil;
    du_write_handle_legacy := nil;
    du_read_file_dispose := nil;
    du_find_open_legacy := nil;
    du_find_next_legacy := nil;
    du_find_close_legacy := nil;
    du_file_age_legacy := nil;
    du_set_file_date_legacy := nil;
    du_wait_writable_legacy := nil;
    du_math_legacy := nil;
    du_string_to_float_def_legacy := nil;
    du_sha1 := nil;
    du_sha1_file := nil;
    du_spawn := nil;
    du_execute := nil;
    du_open := nil;
    du_once := nil;
    du_shutdown := nil;
    du_reboot := nil;
    du_logout := nil;
    du_lockkey_set := nil;
    du_lockkey_state := nil;
    du_datetime_set := nil;
    du_terminate := nil;
    du_deltree := nil;
  finally
    GCS.Release;
  end;
end;

procedure Check;
begin
  if GLibHandle = NilHandle then
    raise EduLibNotLoaded.CreateFmt(SduLibNotLoaded,
      [IfThen(GLibLastName = '', DU_LIB_NAME, GLibLastName)]);
end;

initialization
  GCS := TCriticalSection.Create;
  TryLoad(DU_LIB_NAME);

finalization
  Unload;
  FreeAndNil(GCS);

end.
