# Legacy math policy with consumer RTL primitives

`du_math_legacy(op, a, b, dec, kind, callback, context, output)` takes Double/f64
inputs and TRoundToRange decimal -20..20. Output remains untouched on failure:
0 success, -1 invalid argument/op/range, -2 callback exception, -3 Rust panic.
The callback is cdecl: `(op:u32, a:f64, b:f64, n:i32, input:i64, float:*f64,
integer:*i64, context:pointer)->i32`. It catches all consumer exceptions and
returns nonzero; caller context stores the original exception class/message.
The consumer rethrows only after Rust has returned, avoiding cross-ABI unwind.

Math policy, control flow, digit adjustments, default/type selection and final
finite sanitization are in Rust. Arithmetic and precision-sensitive primitives
remain callback operations because Delphi intrinsic Frac/Int behavior, CPU
rounding mode and mixed Extended/Double intermediates vary by platform. No
standard Rust round/cast/pow implementation replaces those contracts. The
callbacks preserve Delphi's active rounding mode rather than changing it.

| Math op | Behavior |
|---|---|
| 1 | TruncExt |
| 2 | RoundTruncExt rounding branch, kind preserves ATipoArred |
| 3 | RoundABNT |
| 4 | RoundToExt |
| 5 | DivExt(a,b) |
| 6 | IsFrac, returned as Double 0/1 |
| 7 | RoundTruncExt truncation branch |

Callback operations (independent numeric fragments): 1 IntPower(10.0,n),
2 Frac(Abs(a))>0, 3 Frac, 4 Int, 5 addition, 6 multiply, 7 divide,
8 Trunc(SimpleRoundTo(Frac(a)*100,-9)), 9 CompareValue(a,0,0.0001),
10 SimpleRoundTo(input/Max(a,1),-9), 11 Round(Frac(input/10)*10),
12 CompareValue(Frac(a*10),0,0.0001),
13 Round(SimpleRoundTo(Frac(input/10),-9)*10),
14 Int((a/b)+/-0.05)*b according to a<0,
15 Abs(a)>0, 16 IsZero(a, n=1 ? 0.001 : 0.0001),
17 SameValue(a,b,0.00001), 18 Trunc(a), 19 IsZero(input,0.0001), 20 IntPower(10,n) integer-literal overload.
The original integer10 and real10.0 expressions intentionally remain distinct.

`du_string_to_float_def_legacy(input, len, separator, default, parser, context,
output)` normalizes borrowed UTF16 code units in Rust: Trim characters<=32,
replace dots/commas with the current separator, retain only its last occurrence.
Its cdecl parser `(context, UTF16:pointer, len:usize, out:f64)->i32` calls the
consumer StrToFloat and catches every conversion exception. Parser failure
returns the caller's default, preserving its signed-zero bits. Rust never
substitutes its own float parser. Empty original input returns default without
parsing. Argument/Rust panic errors use -1/-3 and leave output unchanged.

The optional bindings expose `dUtils.LegacyMath` and
`dUtils.LegacyStringToFloatDef`, with the same callback implementations copied
to the standalone Nexo facade. Regression authority is the frozen dUtils oracle
SHA7886e31ea49d96fbce50faa3f0fa2b5e8ec9b720 and installed Delphi13 RTL37.
Windows Extended is Double-size; the observed Linux64 compiler reports
SizeOf(Extended)=16. Exact bits and exception class/message must be compared
under all four consumer rounding modes before deployment.
