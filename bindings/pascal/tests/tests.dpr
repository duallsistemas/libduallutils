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

procedure TestSpawn;
var
  O: Integer;
begin
  Assert(not dUtils.Spawn('blah blah', []));
  Assert(dUtils.Spawn('echo', '', [], [], True, O));
  Assert(O = 0);
end;

procedure TestExecute;
var
  O, E: string;
  C: Integer;
begin
  Assert(not dUtils.Execute('blah blah', [], O));
{$IFDEF MSWINDOWS}
  Assert(dUtils.Execute('cmd', '', ['/C', 'echo %foo%'], ['foo=bar'], O, E, C));
{$ELSE}
  Assert(dUtils.Execute('sh', '', ['-c', 'echo $foo'], ['foo=bar'], O, E, C));
{$ENDIF}
  Assert(O.Trim.Equals('bar'));
  Assert(C = 0);
end;

begin
  dUtils.Load(Concat('../../target/release/', dUtils.LIB_NAME));
  TestVersion;
  TestMD5;
  TestSHA1;
  TestSpawn;
  TestExecute;
  Writeln('All tests passed!');
{$IFDEF MSWINDOWS}
  Writeln('Press ENTER to exit ...');
  Readln;
{$ENDIF}
end.
