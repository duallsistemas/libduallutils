//! dUtils rounding policy in Rust; platform RTL numeric primitives are callbacks.
use libc::c_void;
use std::slice;

type Callback =
    unsafe extern "C" fn(u32, f64, f64, i32, i64, *mut f64, *mut i64, *mut c_void) -> i32;
type Parser = unsafe extern "C" fn(*mut c_void, *const u16, usize, *mut f64) -> i32;
struct Math {
    callback: Callback,
    context: *mut c_void,
}
impl Math {
    fn call(&self, op: u32, a: f64, b: f64, n: i32, i: i64) -> Result<(f64, i64), i32> {
        let mut value = 0.;
        let mut integer = 0;
        if unsafe { (self.callback)(op, a, b, n, i, &mut value, &mut integer, self.context) } != 0 {
            Err(-2)
        } else {
            Ok((value, integer))
        }
    }
    fn f(&self, op: u32, a: f64, b: f64, n: i32, i: i64) -> Result<f64, i32> {
        Ok(self.call(op, a, b, n, i)?.0)
    }
    fn i(&self, op: u32, a: f64, b: f64, n: i32, i: i64) -> Result<i64, i32> {
        Ok(self.call(op, a, b, n, i)?.1)
    }
    fn is_zero(&self, a: f64, n: i32) -> Result<bool, i32> {
        Ok(self.i(16, a, 0., n, 0)? != 0)
    }
    fn trunc_ext(&self, a: f64, dec: i32) -> Result<f64, i32> {
        let frac = self.f(3, a, 0., 0, 0)?;
        if self.i(15, frac, 0., 0, 0)? == 0 {
            return Ok(a);
        }
        let integer = self.f(4, a, 0., 0, 0)?;
        let factor = self.f(1, 10., 0., -dec.abs(), 0)?;
        let adjusted = self.f(14, frac, factor, 0, 0)?;
        self.f(5, integer, adjusted, 0, 0)
    }
    fn cal_result(&self, i: i64, pow: f64, positive: bool) -> Result<f64, i32> {
        let result = self.f(10, pow, 0., 0, i)?;
        if positive {
            Ok(result)
        } else {
            self.f(6, result, -1., 0, 0)
        }
    }
    fn round_ext(&self, a: f64, dec: i32, kind: i32) -> Result<f64, i32> {
        if self.is_zero(a, 1)? {
            return Ok(0.);
        }
        if self.i(2, a, 0., 0, 0)? == 0 {
            return Ok(a);
        }
        let pow = self.f(20, 10., 0., dec.abs(), 0)?;
        let scaled = self.f(6, f64::from_bits(a.to_bits() & !(1u64 << 63)), pow, 0, 0)?;
        let frac = self.i(8, scaled, 0., 0, 0)?;
        let positive = self.i(9, a, 0., 0, 0)? >= 0;
        let mut integer = self.i(18, scaled, 0., 0, 0)?;
        if frac == 0 {
            return self.cal_result(integer, pow, positive);
        }
        if kind == 4 {
            if self.i(19, 0., 0., 0, integer)? == 0 {
                match self.i(13, 0., 0., 0, integer)? {
                    1 | 6 => integer = integer.wrapping_sub(1),
                    2 => integer = integer.wrapping_sub(2),
                    3 | 7 => integer = integer.wrapping_add(2),
                    4 | 8 => integer = integer.wrapping_add(1),
                    _ => {}
                }
            }
        } else if frac == 50 {
            match kind {
                0 | 1 => integer = integer.wrapping_add(1),
                2 => integer = integer.wrapping_sub(1),
                3 => {
                    if self.i(11, 0., 0., 0, integer)? & 1 != 0 || self.i(12, scaled, 0., 0, 0)? > 0
                    {
                        integer = integer.wrapping_add(1);
                    }
                }
                _ => {}
            }
        } else if frac > 50 {
            integer = integer.wrapping_add(1);
        }
        self.cal_result(integer, pow, positive)
    }
    fn apply(&self, op: u32, a: f64, b: f64, dec: i32, kind: i32) -> Result<f64, i32> {
        match op {
            1 => self.trunc_ext(a, dec),
            2 | 7 => {
                let value = if op == 2 {
                    self.round_ext(a, dec, kind)?
                } else {
                    self.trunc_ext(a, dec)?
                };
                Ok(if value.is_finite() { value } else { 0. })
            }
            3 => self.round_ext(a, dec, 3),
            4 => self.round_ext(a, dec, kind),
            5 => {
                if self.is_zero(a, 2)? || self.is_zero(b, 2)? {
                    return Ok(0.);
                }
                if self.i(17, a, b, 0, 0)? != 0 {
                    return Ok(1.);
                }
                let quotient = self.f(7, a, b, 0, 0)?;
                self.round_ext(quotient, dec, 0)
            }
            6 => Ok(if self.i(2, a, 0., 0, 0)? != 0 { 1. } else { 0. }),
            _ => Err(-1),
        }
    }
}

/// Callback must catch exceptions and return nonzero, retaining caller context.
/// 0 success, -1 invalid arguments/op/dec, -2 RTL error, -3 Rust panic.
/// Output is unchanged on failure. Decimal argument matches TRoundToRange [-20,20].
#[no_mangle]
pub unsafe extern "C" fn du_math_legacy(
    op: u32,
    a: f64,
    b: f64,
    dec: i32,
    kind: i32,
    callback: Option<Callback>,
    context: *mut c_void,
    out: *mut f64,
) -> i32 {
    if out.is_null() || callback.is_none() || !(-20..=20).contains(&dec) {
        return -1;
    }
    match std::panic::catch_unwind(|| {
        Math {
            callback: callback.unwrap(),
            context,
        }
        .apply(op, a, b, dec, kind)
    }) {
        Ok(Ok(value)) => {
            *out = value;
            0
        }
        Ok(Err(code)) => code,
        Err(_) => -3,
    }
}

/// Normalize UTF16 decimal separators, then parse through the consumer RTL.
/// Parser exceptions return default exactly as original StringToFloatDef.
#[no_mangle]
pub unsafe extern "C" fn du_string_to_float_def_legacy(
    input: *const u16,
    len: usize,
    separator: u16,
    default: f64,
    parser: Option<Parser>,
    context: *mut c_void,
    out: *mut f64,
) -> i32 {
    if out.is_null()
        || parser.is_none()
        || (input.is_null() && len != 0)
        || len > isize::MAX as usize / 2
    {
        return -1;
    }
    let result = std::panic::catch_unwind(|| {
        if len == 0 {
            return default;
        }
        let mut input = slice::from_raw_parts(input, len);
        while input.first().map_or(false, |c| *c <= 32) {
            input = &input[1..];
        }
        while input.last().map_or(false, |c| *c <= 32) {
            input = &input[..input.len() - 1];
        }
        let normalized: Vec<u16> = input
            .iter()
            .map(|c| if *c == 46 || *c == 44 { separator } else { *c })
            .collect();
        let last = normalized.iter().rposition(|c| *c == separator);
        let normalized: Vec<u16> = normalized
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                if *c == separator && Some(i) != last {
                    None
                } else {
                    Some(*c)
                }
            })
            .collect();
        let mut value = default;
        if (parser.unwrap())(context, normalized.as_ptr(), normalized.len(), &mut value) == 0 {
            value
        } else {
            default
        }
    });
    match result {
        Ok(value) => {
            *out = value;
            0
        }
        Err(_) => -3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn callback_failures_do_not_change_output() {
        unsafe extern "C" fn failure(
            _: u32,
            _: f64,
            _: f64,
            _: i32,
            _: i64,
            _: *mut f64,
            _: *mut i64,
            _: *mut c_void,
        ) -> i32 {
            1
        }
        let mut out = 123.;
        unsafe {
            assert_eq!(
                du_math_legacy(
                    4,
                    1.234,
                    0.,
                    2,
                    0,
                    Some(failure),
                    std::ptr::null_mut(),
                    &mut out
                ),
                -2
            );
            assert_eq!(out, 123.);
            assert_eq!(
                du_math_legacy(
                    4,
                    1.,
                    0.,
                    21,
                    0,
                    Some(failure),
                    std::ptr::null_mut(),
                    &mut out
                ),
                -1
            );
            assert_eq!(
                du_math_legacy(
                    99,
                    1.,
                    0.,
                    0,
                    0,
                    Some(failure),
                    std::ptr::null_mut(),
                    &mut out
                ),
                -1
            );
        }
    }
    #[test]
    fn rounding_policy_dispatches_ties_modes_and_sanitizer() {
        // Numeric values are scripted: this test verifies Rust decision flow,
        // not a substitute for the consumer's exact RTL differential tests.
        unsafe extern "C" fn scripted(
            op: u32,
            a: f64,
            b: f64,
            _: i32,
            i: i64,
            f: *mut f64,
            v: *mut i64,
            _: *mut c_void,
        ) -> i32 {
            *f = 0.;
            *v = 0;
            match op {
                16 => *v = 0,
                2 => *v = 1,
                1 | 20 => *f = 100.,
                6 => *f = a * b,
                8 => *v = 50,
                9 => *v = 1,
                18 => *v = 123,
                10 => *f = i as f64 / a,
                11 => *v = 1,
                12 => *v = 0,
                13 => *v = 3,
                19 => *v = i64::from(i == 0),
                _ => return 1,
            };
            0
        }
        let mut out = 0.;
        unsafe {
            assert_eq!(
                du_math_legacy(
                    4,
                    1.235,
                    0.,
                    2,
                    0,
                    Some(scripted),
                    std::ptr::null_mut(),
                    &mut out
                ),
                0
            );
            assert_eq!(out, 1.24);
            assert_eq!(
                du_math_legacy(
                    4,
                    1.235,
                    0.,
                    2,
                    2,
                    Some(scripted),
                    std::ptr::null_mut(),
                    &mut out
                ),
                0
            );
            assert_eq!(out, 1.22);
            assert_eq!(
                du_math_legacy(
                    4,
                    1.235,
                    0.,
                    2,
                    3,
                    Some(scripted),
                    std::ptr::null_mut(),
                    &mut out
                ),
                0
            );
            assert_eq!(out, 1.24);
            assert_eq!(
                du_math_legacy(
                    4,
                    1.235,
                    0.,
                    2,
                    4,
                    Some(scripted),
                    std::ptr::null_mut(),
                    &mut out
                ),
                0
            );
            assert_eq!(out, 1.25);
        }
    }
    #[test]
    fn sanitizer_runs_only_after_success_and_keeps_signed_zero() {
        unsafe extern "C" fn nonfraction(
            op: u32,
            a: f64,
            _: f64,
            _: i32,
            _: i64,
            f: *mut f64,
            i: *mut i64,
            _: *mut c_void,
        ) -> i32 {
            *f = a;
            *i = 0;
            if [16, 2, 3, 15].contains(&op) {
                0
            } else {
                1
            }
        }
        let mut out = 123f64;
        unsafe {
            assert_eq!(
                du_math_legacy(
                    2,
                    f64::INFINITY,
                    0.,
                    2,
                    0,
                    Some(nonfraction),
                    std::ptr::null_mut(),
                    &mut out
                ),
                0
            );
            assert_eq!(out.to_bits(), 0f64.to_bits());
            assert_eq!(
                du_math_legacy(
                    7,
                    -0.,
                    0.,
                    2,
                    0,
                    Some(nonfraction),
                    std::ptr::null_mut(),
                    &mut out
                ),
                0
            );
            assert_eq!(out.to_bits(), (-0f64).to_bits());
        }
    }

    #[test]
    fn parser_receives_normalized_units_and_default_signed_zero() {
        unsafe extern "C" fn parser(
            context: *mut c_void,
            data: *const u16,
            len: usize,
            out: *mut f64,
        ) -> i32 {
            *(context as *mut Vec<u16>) = slice::from_raw_parts(data, len).to_vec();
            *out = 9.;
            0
        }
        unsafe extern "C" fn failed(_: *mut c_void, _: *const u16, _: usize, _: *mut f64) -> i32 {
            1
        }
        let input: Vec<u16> = " 1.234,56 ".encode_utf16().collect();
        let mut received: Vec<u16> = vec![];
        let mut out = 0.;
        unsafe {
            assert_eq!(
                du_string_to_float_def_legacy(
                    input.as_ptr(),
                    input.len(),
                    44,
                    -0.,
                    Some(parser),
                    &mut received as *mut _ as *mut c_void,
                    &mut out
                ),
                0
            );
            assert_eq!(received, "1234,56".encode_utf16().collect::<Vec<_>>());
            assert_eq!(out, 9.);
            assert_eq!(
                du_string_to_float_def_legacy(
                    input.as_ptr(),
                    input.len(),
                    44,
                    -0.,
                    Some(failed),
                    std::ptr::null_mut(),
                    &mut out
                ),
                0
            );
            assert_eq!(out.to_bits(), (-0f64).to_bits());
        }
    }
}
