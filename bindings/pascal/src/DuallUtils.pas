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

  { TdUtils }

  TdUtils = packed record
  public const
    LIB_NAME = libduallutils.LIB_NAME;
  public
    class procedure Load(const ALibraryName: TFileName = LIB_NAME); static;
    class procedure Unload; static;
    class function Version: string; static;
    class function MD5(const S: string): string; static;
    class function SHA1(const S: string): string; static;
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

{ TdUtils }

class procedure TdUtils.Load(const ALibraryName: TFileName);
begin
  Unload;
  libduallutils.Load(ALibraryName);
end;

class procedure TdUtils.Unload;
begin
  libduallutils.Unload;
end;

class function TdUtils.Version: string;
begin
  libduallutils.Check;
  Result := TMarshal.ToString(libduallutils.du_version);
end;

class function TdUtils.MD5(const S: string): string;
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

class function TdUtils.SHA1(const S: string): string;
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

initialization

finalization
  TdUtils.Unload;

end.
