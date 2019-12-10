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
  Assert(Length(TdUtils.Version) >= 5);
end;

procedure TestMD5;
begin
  Assert(TdUtils.MD5('abc123') = 'e99a18c428cb38d5f260853678922e03');
end;

procedure TestSHA1;
begin
  Assert(TdUtils.SHA1('abc123') = '6367c48dd193d56ea7b0baad25b19455e529f5ee');
end;

begin
  TdUtils.Load(Concat('../../target/release/', TdUtils.LIB_NAME));
  TestVersion;
  TestMD5;
  TestSHA1;
{$IFDEF MSWINDOWS}
  Writeln('All tests passed!');
  Writeln('Press ENTER to exit ...');
  Readln;
{$ENDIF}
end.
