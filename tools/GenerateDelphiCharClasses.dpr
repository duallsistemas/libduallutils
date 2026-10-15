program GenerateDelphiCharClasses;
{$APPTYPE CONSOLE}
uses System.SysUtils, System.Classes, System.Character;
var I: Integer; C: Char; Flags: array[0..65535] of Byte; F: TFileStream;
begin
  if ParamCount <> 1 then Halt(2);
  for I := 0 to 65535 do
  begin
    C := Char(I);
    Flags[I] := 0;
    if C.IsNumber then Flags[I] := Flags[I] or 1;
    if C.IsLetter then Flags[I] := Flags[I] or 2;
  end;
  F := TFileStream.Create(ParamStr(1), fmCreate);
  try F.WriteBuffer(Flags, SizeOf(Flags)); finally F.Free; end;
end.
