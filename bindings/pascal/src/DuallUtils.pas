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
  SysUtils,
  Marshalling,
  libduallutils;

const
  MD5_SIZE = 32;
  SHA1_SIZE = 40;

resourcestring
  SInvalidFunctionArgument = 'Invalid function argument.';
  SUnknownLibraryError = 'Unknown library error.';

type

  { EdUtils }

  EdUtils = class(EInvalidOpException);

  { dUtils }

  dUtils = packed record
  public const
    LIB_NAME = libduallutils.DU_LIB_NAME;
  public
    class procedure Load(const ALibraryName: TFileName = LIB_NAME); static;
    class procedure Unload; static;
    class function Version: string; static;
    class function MD5(const S: string): string; static;
    class function SHA1(const S: string): string; static;
    class function Spawn(const AProgram: TFileName; const AWorkDir: string;
      const AArgs, AEnvs: array of string;
      AWaiting: Boolean; out AExitCode: Integer): Boolean; overload; static;
    class function Spawn(const AProgram: TFileName; const AArgs: array of string;
      AWaiting: Boolean = False): Boolean; overload; static;
  end;

implementation

procedure RaiseInvalidFunctionArgument; inline;
begin
  raise EdUtils.Create(SInvalidFunctionArgument);
end;

procedure RaiseUnknownLibraryError; inline;
begin
  raise EdUtils.Create(SUnknownLibraryError);
end;

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

class function dUtils.Spawn(const AProgram: TFileName; const AWorkDir: string;
  const AArgs, AEnvs: array of string; AWaiting: Boolean;
  out AExitCode: Integer): Boolean;
var
  M: TMarshaller;
  A, E: array of Pcchar;
  R: cint;
  I: Byte;
begin
  libduallutils.Check;
  I := Length(AArgs);
  if I > 0 then
  begin
    SetLength(A, Succ(I));
    for I := Low(AArgs) to High(AArgs) do
      A[I] := M.ToCString(AArgs[I]);
    A[Length(AArgs)] := nil;
  end;
  I := Length(AEnvs);
  if I > 0 then
  begin
    SetLength(E, Succ(I));
    for I := Low(AEnvs) to High(AEnvs) do
      E[I] := M.ToCString(AEnvs[I]);
    E[Length(AEnvs)] := nil;
  end;
  R := libduallutils.du_spawn(M.ToCString(AProgram),
    M.ToCNullableString(AWorkDir), @A[0], @E[0], AWaiting, @AExitCode);
  case R of
    -1: RaiseInvalidFunctionArgument;
    -2: Exit(False);
    -3: RaiseUnknownLibraryError;
  end;
  Result := True;
end;

class function dUtils.Spawn(const AProgram: TFileName;
  const AArgs: array of string; AWaiting: Boolean): Boolean;
var
  O: Integer;
begin
  Result := dUtils.Spawn(AProgram, '', AArgs, [], AWaiting, O);
end;

initialization

finalization
  dUtils.Unload;

end.
