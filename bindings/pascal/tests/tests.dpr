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
  Classes,
  DuallUtils;

procedure TestVersion;
begin
  Assert(Length(dUtils.Version) >= 5);
end;

procedure TestMD5;
begin
  Assert(dUtils.MD5('abc123') = 'e99a18c428cb38d5f260853678922e03');
end;

procedure TestMD5File;
var
  F: TBytesStream;
begin
  F := TBytesStream.Create(BytesOf('abc123'));
  try
    F.SaveToFile('abc123.txt');
    Assert(dUtils.MD5File('abc123.txt') = 'e99a18c428cb38d5f260853678922e03');
    DeleteFile('abc123.txt');
  finally
    F.Destroy;
  end;
end;

procedure TestSHA1;
begin
  Assert(dUtils.SHA1('abc123') = '6367c48dd193d56ea7b0baad25b19455e529f5ee');
end;

procedure TestSHA1File;
var
  F: TBytesStream;
begin
  F := TBytesStream.Create(BytesOf('abc123'));
  try
    F.SaveToFile('abc123.txt');
    Assert(dUtils.SHA1File('abc123.txt') = '6367c48dd193d56ea7b0baad25b19455e529f5ee');
    DeleteFile('abc123.txt');
  finally
    F.Destroy;
  end;
end;

procedure TestSpawn;
var
  O: Integer;
begin
  Assert(not dUtils.Spawn('blah blah', []));
  Assert(dUtils.Spawn('echo', '', [], [],
{$IFDEF MSWINDOWS}True,{$ENDIF}True, O));
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
  TestMD5File;
  TestSHA1;
  TestSHA1File;
  TestSpawn;
  TestExecute;
  // TestOpen
  Writeln('All tests passed!');
{$IFDEF MSWINDOWS}
  Writeln('Press ENTER to exit ...');
  Readln;
{$ENDIF}
end.
