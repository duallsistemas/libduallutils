program tests;

{$IFDEF FPC}
 {$MODE DELPHI}
{$ENDIF}
{$IFDEF MSWINDOWS}
 {$APPTYPE CONSOLE}
{$ENDIF}
{$ASSERTIONS ON}

uses
  SysUtils,
  DuallUtils;

procedure TestVersion;
begin
  Assert(Length(dUtils.Version) >= 5);
end;

procedure TestMD5;
begin
  Assert(dUtils.MD5('abc123') = 'e99a18c428cb38d5f260853678922e03');
end;

procedure TestSHA1;
begin
  Assert(dUtils.SHA1('abc123') = '6367c48dd193d56ea7b0baad25b19455e529f5ee');
end;

begin
  dUtils.Load(Concat('../../target/release/', dUtils.LIB_NAME));
  TestVersion;
  TestMD5;
  TestSHA1;
  Writeln('All tests passed!');
{$IFDEF MSWINDOWS}
  Writeln('Press ENTER to exit ...');
  Readln;
{$ENDIF}
end.
