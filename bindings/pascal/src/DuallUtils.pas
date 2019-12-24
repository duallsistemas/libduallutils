unit DuallUtils;

{$IFDEF FPC}
 {$MODE DELPHI}
 {$IFDEF VER3_0}
  {$PUSH}{$MACRO ON}
  {$DEFINE EInvalidOpException := Exception}
  {$POP}
 {$ENDIF}
{$ENDIF}

interface

uses
{$IFDEF MSWINDOWS}
  Windows,
  Classes,
{$ENDIF}
  SysUtils,
{$IFNDEF FPC}
  IOUtils,
{$ENDIF}
  Marshalling,
  libduallutils;

const
  MD5_SIZE = 32;
  SHA1_SIZE = 40;

resourcestring
  SInvalidFunctionArgument = 'Invalid function argument.';
  SUnknownErrorInFunction = 'Unknown error in function: %s.';
  SFileNotFound = 'File not found: %s.';
  SOperationNotPermitted = 'Operation not permitted.';

type

  { EdUtils }

  EdUtils = class(EInvalidOpException);

  { TdLockKeys }

  TdLockKeys = (lkCapitalLock = DU_LK_CAPSLOCK, lkNumberLock = DU_LK_NUMLOCK,
    lkScrollingLock = DU_LK_SCROLLLOCK);

  { TdSignals }

  TdSignals = (sigHangup, sigInterrupt, sigQuit, sigIllegal, sigAbort, sigKill,
    sigUser1, sigSegv, sigUser2, sigPipe, sigAlarm, sigTerm);

  { dUtils }

  dUtils = packed record
  public const
    LIB_NAME = libduallutils.DU_LIB_NAME;
  public
    class procedure Load(const ALibraryName: TFileName = LIB_NAME); static;
    class procedure Unload; static;
    class function Version: string; static;
    class function MD5(const S: string): string; static;
    class function TryMD5File(const AFileName: TFileName;
      out AMD5: string): Boolean; static;
    class function MD5File(const AFileName: TFileName): string; static;
    class function SHA1(const S: string): string; static;
    class function TrySHA1File(const AFileName: TFileName;
      out ASHA1: string): Boolean; static;
    class function SHA1File(const AFileName: TFileName): string; static;
    class function Spawn(const AProgram: TFileName; const AWorkDir: string;
      const AArgs, AEnvs: array of string; {$IFDEF MSWINDOWS}AHidden,{$ENDIF}
      AWaiting: Boolean; out AExitCode: Integer): Boolean; overload; static;
    class function Spawn(const AProgram: TFileName; const AArgs: array of string;
      {$IFDEF MSWINDOWS}AHidden: Boolean = False;{$ENDIF}
      AWaiting: Boolean = False): Boolean; overload; static;
    class function Execute(const AProgram: TFileName; const AWorkDir: string;
      const AArgs, AEnvs: array of string; out AOutput, AError: string;
      out AExitCode: Integer): Boolean; overload; static;
    class function Execute(const AProgram: TFileName; const AArgs: array of string;
      out AOutput: string): Boolean; overload; static;
    class procedure Open(const AFileName: TFileName); static;
    class function Once(const AIdent: string): Boolean; static;
    class function TryShutdown(AForced: Boolean;
      out AErrorMsg: string): Boolean; static;
    class function TryReboot(AForced: Boolean;
      out AErrorMsg: string): Boolean; static;
    class function TryLogout(AForced: Boolean;
      out AErrorMsg: string): Boolean; static;
    class procedure Shutdown(AForced: Boolean = False); static;
    class procedure Reboot(AForced: Boolean = False); static;
    class procedure Logout(AForced: Boolean = False); static;
    class procedure SetLockKey(AKey: TdLockKeys; AEnabled: Boolean); static;
    class function LockKeyState(AKey: TdLockKeys): Boolean; static;
    class procedure SetDateTime(AYear, AMonth, ADay, AHour, AMinute,
      ASecond: Word); overload; static;
    class procedure SetDateTime(const ADateTime: TDateTime); overload; static;
    class function KillAll(const AProcessName: string;
      ASignal: TdSignals = sigTerm): Boolean; overload; static;
    class function DelTree(const APattern: string): Boolean; static;
  end;

implementation

procedure RaiseInvalidFunctionArgument; inline;
begin
  raise EdUtils.Create(SInvalidFunctionArgument);
end;

procedure RaiseUnknownErrorInFunction(const AFuncName: string); inline;
begin
  raise EdUtils.CreateFmt(SUnknownErrorInFunction, [AFuncName]);
end;

function ArrayToCArray(const AArray: array of string;
  out AOutput: TArray<Pcchar>): PPcchar;
var
  M: TMarshaller;
  I: Integer;
begin
  I := Length(AArray);
  if I = 0 then
    Exit(nil);
  SetLength(AOutput, Succ(I));
  for I := Low(AArray) to High(AArray) do
    AOutput[I] := M.ToCString(AArray[I]);
  AOutput[Length(AArray)] := nil;
  Result := @AOutput[0];
end;

{$IFDEF MSWINDOWS}

function ArrayToString(const ASeparator: string;
  const AArray: array of string): string;
var
  I, L: Integer;
begin
  L := Length(AArray);
  if L = 0 then
    Exit(EmptyStr);
  Result := AArray[0];
  for I := 1 to Pred(L) do
    Result := Concat(Result, ASeparator, AArray[I]);
end;

function BuildEnvBlock(const AArray: array of string): TBytes;
var
  M: TBytesStream;
  S: string;
  C: Char;
begin
  M := TBytesStream.Create;
  try
    C := #0;
    for S in AArray do
    begin
{$IFDEF FPC}
      M.Write(S[1], Length(S));
{$ELSE}
      M.Write(TEncoding.Unicode.GetBytes(S), TEncoding.Unicode.GetByteCount(S));
{$ENDIF}
      M.Write(C, SizeOf(Char));
    end;
    M.Write(C, SizeOf(Char));
    Result := M.Bytes;
    SetLength(Result, M.Size);
  finally
    M.Destroy;
  end;
end;

{$ENDIF}

{ dUtils }

class procedure dUtils.Load(const ALibraryName: TFileName);
begin
  Unload;
  libduallutils.Load(ALibraryName);
end;

class procedure dUtils.Unload;
begin
  libduallutils.Unload;
end;

class function dUtils.Version: string;
begin
  libduallutils.Check;
  Result := TMarshal.ToString(libduallutils.du_version);
end;

class function dUtils.MD5(const S: string): string;
var
  M: TMarshaller;
  A: array[0..MD5_SIZE] of cchar;
begin
  libduallutils.Check;
  A[0] := 0;
  if libduallutils.du_md5(M.ToCString(S), @A[0], SizeOf(A)) = -1 then
    RaiseInvalidFunctionArgument;
  Result := TMarshal.ToString(@A[0]);
end;

class function dUtils.TryMD5File(const AFileName: TFileName;
  out AMD5: string): Boolean;
var
  M: TMarshaller;
  A: array[0..MD5_SIZE] of cchar;
  R: cint;
begin
  libduallutils.Check;
  A[0] := 0;
  R := libduallutils.du_md5_file(M.ToCString(AFileName), @A[0], SizeOf(A));
  case R of
    -1: RaiseInvalidFunctionArgument;
    -2: Exit(False);
    -3: RaiseUnknownErrorInFunction('dUtils.MD5File');
  end;
  AMD5 := TMarshal.ToString(@A[0]);
  Result := True;
end;

class function dUtils.MD5File(const AFileName: TFileName): string;
begin
  if not TryMD5File(AFileName, Result) then
    raise EFileNotFoundException.CreateFmt(SFileNotFound, [AFileName]);
end;

class function dUtils.SHA1(const S: string): string;
var
  M: TMarshaller;
  A: array[0..SHA1_SIZE] of cchar;
begin
  libduallutils.Check;
  A[0] := 0;
  if libduallutils.du_sha1(M.ToCString(S), @A[0], SizeOf(A)) = -1 then
    RaiseInvalidFunctionArgument;
  Result := TMarshal.ToString(@A[0]);
end;

class function dUtils.TrySHA1File(const AFileName: TFileName;
  out ASHA1: string): Boolean;
var
  M: TMarshaller;
  A: array[0..SHA1_SIZE] of cchar;
  R: cint;
begin
  libduallutils.Check;
  A[0] := 0;
  R := libduallutils.du_sha1_file(M.ToCString(AFileName), @A[0], SizeOf(A));
  case R of
    -1: RaiseInvalidFunctionArgument;
    -2: Exit(False);
    -3: RaiseUnknownErrorInFunction('dUtils.SHA1File');
  end;
  ASHA1 := TMarshal.ToString(@A[0]);
  Result := True;
end;

class function dUtils.SHA1File(const AFileName: TFileName): string;
begin
  if not TrySHA1File(AFileName, Result) then
    raise EFileNotFoundException.CreateFmt(SFileNotFound, [AFileName]);
end;

class function dUtils.Spawn(const AProgram: TFileName; const AWorkDir: string;
  const AArgs, AEnvs: array of string; {$IFDEF MSWINDOWS}AHidden,{$ENDIF}
  AWaiting: Boolean; out AExitCode: Integer): Boolean;
var
{$IFDEF MSWINDOWS}
  SI: TStartupInfo;
  PI: TProcessInformation;
  C: string;
  W: PChar;
  B: TBytes;
  F: DWORD;
{$ENDIF}
  M: TMarshaller;
  A, E: TArray<Pcchar>;
  R: cint;
begin
  libduallutils.Check;
{$IFDEF MSWINDOWS}
  if AHidden then
  begin
    SI := Default(TStartupInfo);
    SI.cb := SizeOf(TStartupInfo);
    SI.dwFlags := STARTF_USESHOWWINDOW;
    SI.wShowWindow := SW_HIDE;
    PI := Default(TProcessInformation);
    C := AProgram;
    if Length(AArgs) > 0 then
      C := Concat(AProgram, ' ', ArrayToString(' ', AArgs));
    if AWorkDir.IsEmpty then
      W := nil
    else
      W := PChar(AWorkDir + #0);
    if Length(AEnvs) > 0 then
      B := BuildEnvBlock(AEnvs)
    else
      B := nil;
    F := CREATE_NEW_CONSOLE or NORMAL_PRIORITY_CLASS
{$IFNDEF FPC}or CREATE_UNICODE_ENVIRONMENT{$ENDIF};
    if CreateProcess(nil, PChar(C + #0), nil, nil, False, F, B, W, SI, PI) then
    try
      if WaitForSingleObject(PI.hProcess, INFINITE) <> WAIT_FAILED then
      begin
        AExitCode := 0;
        GetExitCodeProcess(PI.hProcess, DWORD(AExitCode));
        Exit(True);
      end;
    finally
      CloseHandle(PI.hProcess);
      CloseHandle(PI.hThread);
    end;
    if GetLastError = ERROR_FILE_NOT_FOUND then
      Exit(False);
    RaiseUnknownErrorInFunction('dUtils.Spawn');
  end;
{$ENDIF}
  R := libduallutils.du_spawn(M.ToCString(AProgram),
    M.ToCNullableString(AWorkDir), ArrayToCArray(AArgs, A),
    ArrayToCArray(AEnvs, E), AWaiting, @AExitCode);
  case R of
    -1: RaiseInvalidFunctionArgument;
    -2: Exit(False);
    -3: RaiseUnknownErrorInFunction('dUtils.Spawn');
  end;
  Result := True;
end;

class function dUtils.Spawn(const AProgram: TFileName;
  const AArgs: array of string; {$IFDEF MSWINDOWS}AHidden,{$ENDIF}
  AWaiting: Boolean): Boolean;
var
  O: Integer;
begin
  Result := dUtils.Spawn(AProgram, '', AArgs, [],
{$IFDEF MSWINDOWS}AHidden,{$ENDIF} AWaiting, O);
end;

class function dUtils.Execute(const AProgram: TFileName; const AWorkDir: string;
  const AArgs, AEnvs: array of string; out AOutput, AError: string;
  out AExitCode: Integer): Boolean;
var
  M: TMarshaller;
  A, E: TArray<Pcchar>;
  SO, SE: Pcchar;
  R: cint;
begin
  libduallutils.Check;
  R := libduallutils.du_execute(M.ToCString(AProgram),
    M.ToCNullableString(AWorkDir), ArrayToCArray(AArgs, A),
    ArrayToCArray(AEnvs, E), @SO, @SE, @AExitCode);
  case R of
    -1: RaiseInvalidFunctionArgument;
    -2: Exit(False);
    -3: RaiseUnknownErrorInFunction('dUtils.Execute');
  end;
  AOutput := TMarshal.ToString(SO);
  du_dispose(SO);
  AError := TMarshal.ToString(SE);
  du_dispose(SE);
  Result := True;
end;

class function dUtils.Execute(const AProgram: TFileName;
  const AArgs: array of string; out AOutput: string): Boolean;
var
  E: string;
  C: Integer;
begin
  Result := dUtils.Execute(AProgram, '', AArgs, [], AOutput, E, C);
  if AOutput.IsEmpty and not E.IsEmpty then
    AOutput := E;
end;

class procedure dUtils.Open(const AFileName: TFileName);
var
  M: TMarshaller;
begin
  libduallutils.Check;
  case libduallutils.du_open(M.ToCString(AFileName)) of
    -1: RaiseInvalidFunctionArgument;
    -2: RaiseUnknownErrorInFunction('dUtils.Open');
  end;
end;

class function dUtils.Once(const AIdent: string): Boolean;
var
  M: TMarshaller;
begin
  libduallutils.Check;
  case libduallutils.du_once(M.ToCNullableString(
{$IFNDEF MSWINDOWS}
    Concat(
{$IFDEF FPC}
      GetTempDir
{$ELSE}
      TPath.GetTempPath
{$ENDIF}
      , '.',
      ChangeFileExt(
{$ENDIF}
        AIdent
{$IFNDEF MSWINDOWS}
        , '-lock'
      )
    )
{$ENDIF}
  )) of
    -1: RaiseInvalidFunctionArgument;
    -2: Exit(False);
    -3: RaiseUnknownErrorInFunction('dUtils.Once');
  end;
  Result := True;
end;

class function dUtils.TryShutdown(AForced: Boolean;
  out AErrorMsg: string): Boolean;
var
  E: array[0..Pred(DU_ERR_SIZE)] of cchar;
  R: cint;
begin
  libduallutils.Check;
  E[0] := 0;
  R := libduallutils.du_shutdown(AForced, @E[0], DU_ERR_SIZE);
  case R of
    0: Exit(True);
    -1: RaiseInvalidFunctionArgument;
    -2: AErrorMsg := TMarshal.ToString(@E[0]);
  end;
  Result := False;
end;

class function dUtils.TryReboot(AForced: Boolean;
  out AErrorMsg: string): Boolean;
var
  E: array[0..Pred(DU_ERR_SIZE)] of cchar;
  R: cint;
begin
  libduallutils.Check;
  R := libduallutils.du_reboot(AForced, @E[0], DU_ERR_SIZE);
  case R of
    0: Exit(True);
    -1: RaiseInvalidFunctionArgument;
    -2: AErrorMsg := TMarshal.ToString(@E[0]);
  end;
  Result := False;
end;

class function dUtils.TryLogout(AForced: Boolean;
  out AErrorMsg: string): Boolean;
var
  E: array[0..Pred(DU_ERR_SIZE)] of cchar;
  R: cint;
begin
  libduallutils.Check;
  R := libduallutils.du_logout(AForced, @E[0], DU_ERR_SIZE);
  case R of
    0: Exit(True);
    -1: RaiseInvalidFunctionArgument;
    -2: AErrorMsg := TMarshal.ToString(@E[0]);
  end;
  Result := False;
end;

class procedure dUtils.Shutdown(AForced: Boolean);
var
  E: string;
begin
  if not TryShutdown(AForced, E) then
    raise EOSError.Create(E);
end;

class procedure dUtils.Reboot(AForced: Boolean);
var
  E: string;
begin
  if not TryReboot(AForced, E) then
    raise EOSError.Create(E);
end;

class procedure dUtils.Logout(AForced: Boolean);
var
  E: string;
begin
  if not TryLogout(AForced, E) then
    raise EOSError.Create(E);
end;

class procedure dUtils.SetLockKey(AKey: TdLockKeys; AEnabled: Boolean);
begin
  libduallutils.Check;
  libduallutils.du_lockkey_set(DU_LOCKKEY(AKey), AEnabled);
end;

class function dUtils.LockKeyState(AKey: TdLockKeys): Boolean;
begin
  libduallutils.Check;
  Result := libduallutils.du_lockkey_state(DU_LOCKKEY(AKey));
end;

class procedure dUtils.SetDateTime(AYear, AMonth, ADay, AHour, AMinute,
  ASecond: Word);
var
  R: cint;
begin
  libduallutils.Check;
  R := libduallutils.du_datetime_set(AYear, AMonth, ADay, AHour, AMinute,
    ASecond);
  case R of
    -1: RaiseInvalidFunctionArgument;
    -2: raise EInvalidOpException.Create(SOperationNotPermitted);
  end;
end;

class procedure dUtils.SetDateTime(const ADateTime: TDateTime);
var
  Y, M, D, H, N, S, MS: Word;
begin
  DecodeDate(ADateTime, Y, M, D);
  DecodeTime(ADateTime, H, N, S, MS);
  dUtils.SetDateTime(Y, M, D, H, N, S);
end;

class function dUtils.KillAll(const AProcessName: string;
  ASignal: TdSignals): Boolean;
var
  M: TMarshaller;
  R: cint;
begin
  libduallutils.Check;
  R := libduallutils.du_killall(M.ToCNullableString(AProcessName),
    DU_SIGNALS(ASignal));
  case R of
    -1: RaiseInvalidFunctionArgument;
    -2: Exit(False);
    -3: raise EInvalidOpException.Create(SOperationNotPermitted);
    -4: RaiseLastOSError;
  end;
  Result := True;
end;

class function dUtils.DelTree(const APattern: string): Boolean;
var
  M: TMarshaller;
begin
  libduallutils.Check;
  Result := libduallutils.du_deltree(M.ToCNullableString(APattern)) = 0;
end;

end.
