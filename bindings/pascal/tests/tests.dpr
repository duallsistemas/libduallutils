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
  Assert(dUtils.MD5('abc123').Equals('e99a18c428cb38d5f260853678922e03'));
end;

procedure TestTryMD5File;
var
  F: TBytesStream;
  O: string;
begin
  Assert(not dUtils.TryMD5File('blah blah', O));
  Assert(O.IsEmpty);
  F := TBytesStream.Create(BytesOf('abc123'));
  try
    F.SaveToFile('abc123.txt');
    Assert(dUtils.TryMD5File('abc123.txt', O));
    Assert(O.Equals('e99a18c428cb38d5f260853678922e03'));
    DeleteFile('abc123.txt');
  finally
    F.Destroy;
  end;
end;

procedure TestMD5File;
var
  F: TBytesStream;
begin
  F := TBytesStream.Create(BytesOf('abc123'));
  try
    F.SaveToFile('abc123.txt');
    Assert(dUtils.MD5File('abc123.txt').Equals('e99a18c428cb38d5f260853678922e03'));
    DeleteFile('abc123.txt');
  finally
    F.Destroy;
  end;
end;

procedure TestSHA1;
begin
  Assert(dUtils.SHA1('abc123').Equals('6367c48dd193d56ea7b0baad25b19455e529f5ee'));
end;

procedure TestTrySHA1File;
var
  F: TBytesStream;
  O: string;
begin
  Assert(not dUtils.TrySHA1File('blah blah', O));
  Assert(O.IsEmpty);
  F := TBytesStream.Create(BytesOf('abc123'));
  try
    F.SaveToFile('abc123.txt');
    Assert(dUtils.TrySHA1File('abc123.txt', O));
    Assert(O.Equals('6367c48dd193d56ea7b0baad25b19455e529f5ee'));
    DeleteFile('abc123.txt');
  finally
    F.Destroy;
  end;
end;

procedure TestSHA1File;
var
  F: TBytesStream;
begin
  F := TBytesStream.Create(BytesOf('abc123'));
  try
    F.SaveToFile('abc123.txt');
    Assert(dUtils.SHA1File('abc123.txt').Equals('6367c48dd193d56ea7b0baad25b19455e529f5ee'));
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

procedure TestOnce;
begin
  Assert(dUtils.Once);
  Assert(not dUtils.Once);
end;

begin
  dUtils.Load(Concat('../../target/release/', dUtils.LIB_NAME));
  TestVersion;
  TestMD5;
  TestTryMD5File;
  TestMD5File;
  TestSHA1;
  TestTrySHA1File;
  TestSHA1File;
  TestSpawn;
  TestExecute;
  // TestOpen
  TestOnce;
  Writeln('All tests passed!');
{$IFDEF MSWINDOWS}
  Writeln('Press ENTER to exit ...');
  Readln;
{$ENDIF}
end.
