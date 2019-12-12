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
    class function MD5File(const AFileName: TFileName): string; static;
    class function SHA1(const S: string): string; static;
    class function SHA1File(const AFileName: TFileName): string; static;
    class function Spawn(const AProgram: TFileName; const AWorkDir: string;
      const AArgs, AEnvs: array of string; AWaiting: Boolean;
      out AExitCode: Integer): Boolean; overload; static;
    class function Spawn(const AProgram: TFileName; const AArgs: array of string;
      AWaiting: Boolean = False): Boolean; overload; static;
    class function Execute(const AProgram: TFileName; const AWorkDir: string;
      const AArgs, AEnvs: array of string; out AOutput, AError: string;
      out AExitCode: Integer): Boolean; overload; static;
    class function Execute(const AProgram: TFileName; const AArgs: array of string;
      out AOutput: string): Boolean; overload; static;
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

function ArrayToCArray(const AArray: array of string;
  out AOutput: TArray<Pcchar>): PPChar;
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

class function dUtils.MD5File(const AFileName: TFileName): string;
var
  M: TMarshaller;
  A: array[0..MD5_SIZE] of cchar;
begin
  libduallutils.Check;
  A[0] := 0;
  if libduallutils.du_md5_file(M.ToCString(AFileName),
    @A[0], SizeOf(A)) = -1 then
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

class function dUtils.SHA1File(const AFileName: TFileName): string;
var
  M: TMarshaller;
  A: array[0..SHA1_SIZE] of cchar;
begin
  libduallutils.Check;
  A[0] := 0;
  if libduallutils.du_sha1_file(M.ToCString(AFileName),
    @A[0], SizeOf(A)) = -1 then
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
begin
  libduallutils.Check;
  R := libduallutils.du_spawn(M.ToCString(AProgram),
    M.ToCNullableString(AWorkDir), ArrayToCArray(AArgs, A),
    ArrayToCArray(AEnvs, E), AWaiting, @AExitCode);
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

class function dUtils.Execute(const AProgram: TFileName; const AWorkDir: string;
  const AArgs, AEnvs: array of string; out AOutput, AError: string;
  out AExitCode: Integer): Boolean;
var
  M: TMarshaller;
  A, E: array of Pcchar;
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
    -3: RaiseUnknownLibraryError;
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

end.
