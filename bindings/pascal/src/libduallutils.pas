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

  EduLibNotLoaded = class(EFileNotFoundException);

var
  du_version: function: Pcchar; cdecl;
  du_dispose: procedure(s: Pcchar); cdecl;
  du_md5: function(const str: Pcchar; md5: Pcchar; size: csize_t): cint; cdecl;
  du_sha1: function(const str: Pcchar; sha1: Pcchar; size: csize_t): cint; cdecl;
  du_spawn: function(const &program: Pcchar; const workdir: Pcchar;
    const args: PPcchar; const envs: PPcchar; waiting: cbool;
    exitcode: Pcint): cint; cdecl;
  du_execute: function(const &program: Pcchar; const workdir: Pcchar;
    const args: PPcchar; const envs: PPcchar; output: PPcchar;
    error: PPcchar; exitcode: Pcint): cint; cdecl;

procedure Load(const ALibraryName: TFileName);

procedure Unload;

procedure Check;

implementation

var
  GCS: TCriticalSection;
  GLibHandle: TLibHandle = NilHandle;
  GLibLastName: TFileName = '';

procedure Load(const ALibraryName: TFileName);
begin
  GCS.Acquire;
  try
    if ALibraryName = '' then
      raise EArgumentException.Create(SduLibEmptyName);
    GLibHandle := SafeLoadLibrary(ALibraryName);
    if GLibHandle = NilHandle then
    begin
{$IFDEF MSWINDOWS}
      if GetLastError = ERROR_BAD_EXE_FORMAT then
        raise EduLibNotLoaded.CreateFmt(SduLibInvalid, [ALibraryName]);
{$ENDIF}
      raise EduLibNotLoaded.CreateFmt(SduLibNotLoaded, [ALibraryName])
    end;
    GLibLastName := ALibraryName;

    du_version := GetProcAddress(GLibHandle, 'du_version');
    du_dispose := GetProcAddress(GLibHandle, 'du_dispose');
    du_md5 := GetProcAddress(GLibHandle, 'du_md5');
    du_sha1 := GetProcAddress(GLibHandle, 'du_sha1');
    du_spawn := GetProcAddress(GLibHandle, 'du_spawn');
    du_execute := GetProcAddress(GLibHandle, 'du_execute');
  finally
    GCS.Release;
  end;
end;

procedure Unload;
begin
  GCS.Acquire;
  try
    if GLibHandle = NilHandle then
      Exit;
    if not FreeLibrary(GLibHandle) then
      Exit;
    GLibHandle := NilHandle;
    GLibLastName := '';

    du_version := nil;
    du_dispose := nil;
    du_md5 := nil;
    du_sha1 := nil;
    du_spawn := nil;
    du_execute := nil;
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

finalization
  FreeAndNil(GCS);

end.
