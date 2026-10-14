# UTF-16 legacy text API

`du_legacy_text(op, a, alen, b, blen, n, c, output, capacity, required)` uses
UTF-16 code units with explicit lengths, including embedded NUL and unpaired
surrogates. No terminator is written. Query with null output and zero capacity.
A copy call needs at least `required` units; insufficient capacity returns -1
and leaves the output untouched. Inputs are borrowed and must remain valid for
the call. A null input is accepted only with length zero. `required` must point
to writable, non-overlapping storage. Non-null pointers must be aligned and
valid for their lengths.

Status: 0 success, -1 invalid argument/op/capacity, -2 integer conversion,
-3 caught Rust panic. Boolean results are `1`/`0`;
counts are decimal strings. Delphi bindings provide `dUtils.LegacyText` and
`dUtils.LegacyReplace`; the latter wraps `du_legacy_replace` with three explicit
UTF-16 inputs and the same query/copy output contract. Its replacement is
case-sensitive and non-overlapping; an empty search preserves the input.

| Op | Contract |
|---|---|
| 1 | PadLeft(a, n, c); negative width preserves input |
| 2 | PadRight(a, n, c); negative width preserves input |
| 3 | PadCenter(a, n, c); negative width empty |
| 4 | CountStr(a, b), overlapping matches |
| 5 | RemoveString: source a, substring b |
| 6 | TirarAcentos, original Latin-1 ranges |
| 7 | RemoveZerosEsquerda; then Trim code units <=32 |
| 8 | RemoveZerosDireita; decimal separator c |
| 9 | StrToBoolExt |
| 10 | StrInArray: b contains allowed code units |
| 11 | FindDelimiterInText; empty b defaults to ;,pipe |
| 12 | dDequoteString; quote c, escape Word(n) |
| 13 | dDequoteDigestString result |
| 14 | dDequoteDigestString remaining input |
| 15 | UpCaseBR(Byte(n)); returns one code unit |
| 16 | GerarNomeComponente |
| 17 | RemoveCharInvalid; removes apostrophe, pipe, U+00B0 |
| 18 | Poem_Zeros(a,n) |
| 19 | Fetch prefix (case sensitive delimiter b) |
| 20 | Fetch remaining input (delimiter b) |
| 21 | CalcModulo11; ASCII digits, wrapping i32 sum |
| 22 | OnlyNumber |
| 23 | OnlyAlpha |
| 24 | OnlyAlphaNum |
| 25 | StrIsNumber; n!=0 means numbers only |
| 26 | StrIsAlpha |
| 27 | StrIsAlphaNum; empty true |
| 28 | CharIsNum(c) |
| 29 | CharIsAlpha(c) |
| 30 | CharIsAlphaNum(c) |
| 31 | CalcModulo10 |
| 32 | GetModulo10Dig |
| 33 | Reserved; CompareCNPJBase uses native filter/pad plus Delphi SameText |
| 34 | StrIsIP |

Classification provenance and reproduction: `tools/README.md`.

| 36 | Byte units to uppercase hex; a units must be 0..255 |
| 37 | Hex to byte units; even length; unsigned hex pairs only |
| 38 | SimpleEncrypt byte units; Word(n) key; lowercase hex |
| 39 | SimpleDecrypt hex; Word(n) key; invalid/odd input empty; output byte units |

Encoding remains in the Delphi facade using its TEncoding.UTF8 contract.

| 40 | Base64 encode byte units; n line width, b separator; breaks only after full 3-byte blocks |
| 41 | Base64 decode low byte of each unit; nonalphabet/padding ignored; output byte units |

The Base64 implementation was independently derived from the installed Delphi
13 RTL behavior in System.NetEncoding.pas. Exactly 57 input bytes with width76
and CRLF produces a trailing CRLF; padding a partial block does not trigger
another break. Decode does not reset its bit accumulator at '='.
