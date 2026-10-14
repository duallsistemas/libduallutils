# Delphi character-class table

`src/delphi_char_classes.bin` contains 65,536 bytes, indexed by UTF-16 code
unit. Bit 0 is Delphi `Char.IsNumber`; bit 1 is `Char.IsLetter`. Other bits
are zero. Surrogates are classified independently, as in the original dUtils
loop. Rust does not substitute its own Unicode version.

Generated on 2026-10-14 with Delphi 13 Win64 compiler 37.0 using
`tools/GenerateDelphiCharClasses.dpr`, toolchain located by the Nexo canonical
`scripts/Resolve-DelphiToolchain.ps1`; `rsvars.bat` was loaded before dcc64.

SHA-256: `066f2c222d4cae2a735323f21ba89bd1ec75de8b0598a798518ee82ec3f2e9f7`.

Reproduce after resolving the toolchain and loading its rsvars:

```
dcc64 -B -NSSystem -Etools -N0tools tools\GenerateDelphiCharClasses.dpr
tools\GenerateDelphiCharClasses.exe src\delphi_char_classes.bin
```

Do not regenerate with another RTL version without checking all 65,536 code
units against the consumer's Delphi RTL.
