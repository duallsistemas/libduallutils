//! UTF-16 code-unit contracts from dUtils; no Unicode normalization or UTF-8 conversion.
use std::ptr;
use std::slice;
const CLASSES: &[u8; 65536] = include_bytes!("delphi_char_classes.bin");
fn number(c: u16) -> bool {
    CLASSES[c as usize] & 1 != 0
}
fn letter(c: u16) -> bool {
    CLASSES[c as usize] & 2 != 0
}
fn alpha_num(c: u16) -> bool {
    CLASSES[c as usize] & 3 != 0
}
fn trim(mut a: &[u16]) -> &[u16] {
    while a.first().map_or(false, |c| *c <= 32) {
        a = &a[1..];
    }
    while a.last().map_or(false, |c| *c <= 32) {
        a = &a[..a.len() - 1];
    }
    a
}
fn ascii(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn boolean(b: bool) -> Vec<u16> {
    vec![if b { 49 } else { 48 }]
}
fn accent(c: u16) -> u16 {
    match c {
        0xc0..=0xc5 => 65,
        0xc7 => 67,
        0xc8..=0xcb => 69,
        0xcc..=0xcf => 73,
        0xd1 => 78,
        0xd2..=0xd6 | 0xd8 => 79,
        0xd9..=0xdc => 85,
        0xdd => 89,
        0xe0..=0xe5 => 97,
        0xe7 => 99,
        0xe8..=0xeb => 101,
        0xec..=0xef => 105,
        0xf1 => 110,
        0xf2..=0xf6 | 0xf8 => 111,
        0xf9..=0xfc => 117,
        0xfd | 0xff => 121,
        _ => c,
    }
}
fn pad(a: &[u16], n: i32, c: u16, left: bool) -> Result<Vec<u16>, i32> {
    if n < 0 {
        return Ok(a.to_vec());
    }
    let count = (n as usize).saturating_sub(a.len());
    let mut result = Vec::with_capacity(a.len() + count);
    if left {
        result.resize(count, c);
    }
    result.extend_from_slice(a);
    if !left {
        result.resize(a.len() + count, c);
    }
    Ok(result)
}
fn pos(a: &[u16], b: &[u16]) -> Option<usize> {
    if b.is_empty() || b.len() > a.len() {
        None
    } else {
        a.windows(b.len()).position(|v| v == b)
    }
}
fn dequote_end(a: &[u16], quote: u16, escape: u16) -> usize {
    let mut i = 1;
    while i < a.len() {
        if a[i] == quote {
            break;
        }
        if a[i] == escape {
            i += 1;
        }
        i += 1;
    }
    i
}
fn digit(c: u16) -> Result<i32, i32> {
    if (48..=57).contains(&c) {
        Ok(i32::from(c - 48))
    } else {
        Err(-2)
    }
}
fn modulo10(a: &[u16]) -> Result<Vec<u16>, i32> {
    if a.is_empty() {
        return Ok(vec![]);
    }
    let mut t: Vec<u16> = a.iter().copied().filter(|c| number(*c)).collect();
    let mut sum = 0i32;
    for (i, c) in t.iter().enumerate() {
        let weight = if (t.len() - i) % 2 == 0 { 1 } else { 3 };
        sum = sum.wrapping_add(digit(*c)?.wrapping_mul(weight));
    }
    let rem = sum % 10;
    t.extend(ascii(&if rem == 0 { 0 } else { 10 - rem }.to_string()));
    Ok(t)
}
fn hex_digit(c: u16) -> Option<i32> {
    match c {
        48..=57 => Some(i32::from(c - 48)),
        65..=70 => Some(i32::from(c - 65 + 10)),
        97..=102 => Some(i32::from(c - 97 + 10)),
        _ => None,
    }
}
fn hex_pair(pair: &[u16]) -> Result<u16, i32> {
    if pair.len() != 2 {
        return Err(-2);
    }
    Ok((hex_digit(pair[0]).ok_or(-2)? * 16 + hex_digit(pair[1]).ok_or(-2)?) as u16)
}
fn byte_hex(a: &[u16], lower: bool) -> Result<Vec<u16>, i32> {
    let digits = if lower {
        b"0123456789abcdef"
    } else {
        b"0123456789ABCDEF"
    };
    let mut r = Vec::with_capacity(a.len() * 2);
    for c in a {
        if *c > 255 {
            return Err(-2);
        }
        r.push(u16::from(digits[(*c >> 4) as usize]));
        r.push(u16::from(digits[(*c & 15) as usize]));
    }
    Ok(r)
}
fn ip(a: &[u16]) -> bool {
    let parts: Vec<&[u16]> = a.split(|c| *c == 46).collect();
    parts.len() == 4
        && parts.iter().all(|s| {
            if s.is_empty() {
                return false;
            }
            let mut v = 0u32;
            for c in *s {
                if !(48..=57).contains(c) {
                    return false;
                }
                v = match v
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(u32::from(*c - 48)))
                {
                    Some(v) => v,
                    None => return false,
                };
                if v > 255 {
                    return false;
                }
            }
            true
        })
}
fn apply(op: u32, a: &[u16], b: &[u16], n: i32, c: u16) -> Result<Vec<u16>, i32> {
    Ok(match op {
        1 => return pad(a, n, c, true),
        2 => return pad(a, n, c, false),
        3 => {
            if n <= 0 {
                vec![]
            } else if a.len() >= n as usize {
                a[..n as usize].to_vec()
            } else {
                let mut r = vec![c; (n as usize - a.len()) / 2];
                r.extend_from_slice(a);
                r.resize(n as usize, c);
                r
            }
        }
        4 => ascii(
            &if b.is_empty() || b.len() > a.len() {
                0
            } else {
                a.windows(b.len()).filter(|v| *v == b).count()
            }
            .to_string(),
        ),
        5 => {
            let mut r = vec![];
            let mut i = 0;
            while i < a.len() {
                if !b.is_empty() && a[i..].starts_with(b) {
                    i += b.len();
                } else {
                    r.push(a[i]);
                    i += 1;
                }
            }
            r
        }
        6 => a.iter().map(|c| accent(*c)).collect(),
        7 => {
            let mut i = 0;
            while i + 1 < a.len() && a[i] == 48 {
                i += 1;
            }
            trim(&a[i..]).to_vec()
        }
        8 => {
            if a == [48] {
                return Ok(a.to_vec());
            }
            let mut i = a.len();
            if a.contains(&c) {
                while i > 0 && a[i - 1] == 48 && !(i > 1 && a[i - 2] == c) {
                    i -= 1;
                }
            }
            a[..i].to_vec()
        }
        9 => boolean(
            (1..=4).contains(&a.len()) && [83, 84, 49, 89, 79, 116, 121, 115].contains(&a[0]),
        ),
        10 => boolean(a.iter().all(|c| b.contains(c))),
        11 => {
            let defaults = [59, 44, 124];
            let delimiters = if b.is_empty() { &defaults[..] } else { b };
            vec![delimiters
                .iter()
                .copied()
                .find(|c| *c != 32 && a.contains(c))
                .unwrap_or(32)]
        }
        12 => {
            if a.first() != Some(&c) {
                a.to_vec()
            } else {
                let i = dequote_end(a, c, n as u16);
                a[1..i.min(a.len())].to_vec()
            }
        }
        13 => {
            let i = dequote_end(a, 34, 92);
            if a.is_empty() {
                vec![]
            } else {
                a[1..i.min(a.len())].to_vec()
            }
        }
        14 => {
            let i = dequote_end(a, 34, 92);
            a[(i + 1).min(a.len())..].to_vec()
        }
        15 => {
            let c = (n as u8) as u16;
            vec![if (97..=122).contains(&c) || (222..=255).contains(&c) {
                c ^ 32
            } else {
                c
            }]
        }
        16 => a
            .iter()
            .map(|c| if *c == 32 { 95 } else { accent(*c) })
            .collect(),
        17 => a
            .iter()
            .map(|c| accent(*c))
            .filter(|c| ![39, 124, 0xb0].contains(c))
            .collect(),
        18 => return pad(trim(a), n, 48, true),
        19 => a[..pos(a, b).unwrap_or(a.len())].to_vec(),
        20 => pos(a, b).map_or(vec![], |i| a[i + b.len()..].to_vec()),
        21 => {
            let mut sum = 0i32;
            let mut weight = 2;
            for c in a.iter().rev() {
                sum = sum.wrapping_add(digit(*c)?.wrapping_mul(weight));
                weight = if weight < 9 { weight + 1 } else { 2 };
            }
            let d = 11 - sum % 11;
            let mut r = a.to_vec();
            r.extend(ascii(&if d > 9 { 0 } else { d }.to_string()));
            r
        }
        22 => a.iter().copied().filter(|c| number(*c)).collect(),
        23 => a.iter().copied().filter(|c| letter(*c)).collect(),
        24 => a.iter().copied().filter(|c| alpha_num(*c)).collect(),
        25 => boolean(
            !a.is_empty()
                && a.iter()
                    .all(|c| number(*c) || (n == 0 && [46, 44, 36].contains(c))),
        ),
        26 => boolean(!a.is_empty() && a.iter().all(|c| letter(*c))),
        27 => boolean(a.iter().all(|c| alpha_num(*c))),
        28 => boolean(number(c)),
        29 => boolean(letter(c)),
        30 => boolean(alpha_num(c)),
        31 => return modulo10(a),
        32 => {
            let t = modulo10(a)?;
            t.last().copied().into_iter().collect()
        }
        // 33 reserved: CompareCNPJBase requires Delphi locale-aware SameText.
        34 => boolean(ip(a)),
        36 => return byte_hex(a, false),
        37 => {
            if a.len() % 2 != 0 {
                return Err(-2);
            }
            a.chunks(2).map(hex_pair).collect::<Result<Vec<_>, _>>()?
        }
        38 => {
            let mut key = n as u16;
            let mut bytes = vec![];
            for byte in a {
                if *byte > 255 {
                    return Err(-2);
                }
                let encoded = *byte ^ (key >> 8);
                bytes.push(encoded);
                key = (encoded.wrapping_add(key))
                    .wrapping_mul(53761)
                    .wrapping_add(32618);
            }
            return byte_hex(&bytes, true);
        }
        39 => {
            if a.len() % 2 != 0 {
                return Ok(vec![]);
            }
            let mut bytes = match a.chunks(2).map(hex_pair).collect::<Result<Vec<_>, _>>() {
                Ok(v) => v,
                Err(_) => return Ok(vec![]),
            };
            let mut key = n as u16;
            for byte in &mut bytes {
                let encoded = *byte;
                *byte ^= key >> 8;
                key = (encoded.wrapping_add(key))
                    .wrapping_mul(53761)
                    .wrapping_add(32618);
            }
            bytes
        }
        40 => return crate::legacy_base64::encode(a, n, b),
        41 => crate::legacy_base64::decode(a),
        _ => return Err(-1),
    })
}

/// Query/copy API. Buffers contain UTF-16 units without a terminator.
/// 0 success, -1 invalid pointer/op/capacity, -2 numeric conversion, -3 panic,
/// Negative padding widths preserve the source. Never writes an undersized destination.
#[no_mangle]
pub unsafe extern "C" fn du_legacy_text(
    op: u32,
    a: *const u16,
    alen: usize,
    b: *const u16,
    blen: usize,
    n: i32,
    c: u16,
    out: *mut u16,
    capacity: usize,
    required: *mut usize,
) -> i32 {
    if required.is_null()
        || (!a.is_null() && alen > isize::MAX as usize / 2)
        || (!b.is_null() && blen > isize::MAX as usize / 2)
        || capacity > isize::MAX as usize / 2
        || (a.is_null() && alen != 0)
        || (b.is_null() && blen != 0)
        || (out.is_null() && capacity != 0)
    {
        return -1;
    }
    *required = 0;
    let result = std::panic::catch_unwind(|| {
        let a = if alen == 0 {
            &[][..]
        } else {
            slice::from_raw_parts(a, alen)
        };
        let b = if blen == 0 {
            &[][..]
        } else {
            slice::from_raw_parts(b, blen)
        };
        apply(op, a, b, n, c)
    });
    match result {
        Ok(Ok(value)) => {
            *required = value.len();
            if out.is_null() {
                return 0;
            }
            if capacity < value.len() {
                return -1;
            }
            if !value.is_empty() {
                ptr::copy(value.as_ptr(), out, value.len());
            }
            0
        }
        Ok(Err(code)) => code,
        Err(_) => -3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn buffer_query_bounds_and_codeunits() {
        unsafe {
            let a = [0xd800, 0, 65];
            let mut required = 99;
            assert_eq!(
                du_legacy_text(
                    1,
                    a.as_ptr(),
                    3,
                    ptr::null(),
                    0,
                    5,
                    32,
                    ptr::null_mut(),
                    0,
                    &mut required
                ),
                0
            );
            assert_eq!(required, 5);
            let mut out = [77u16; 7];
            assert_eq!(
                du_legacy_text(
                    1,
                    a.as_ptr(),
                    3,
                    ptr::null(),
                    0,
                    5,
                    32,
                    out.as_mut_ptr(),
                    4,
                    &mut required
                ),
                -1
            );
            assert_eq!(out, [77; 7]);
            assert_eq!(
                du_legacy_text(
                    1,
                    a.as_ptr(),
                    3,
                    ptr::null(),
                    0,
                    5,
                    32,
                    out.as_mut_ptr(),
                    5,
                    &mut required
                ),
                0
            );
            assert_eq!(out, [32, 32, 0xd800, 0, 65, 77, 77]);
            assert_eq!(
                du_legacy_text(
                    999,
                    ptr::null(),
                    0,
                    ptr::null(),
                    0,
                    0,
                    0,
                    ptr::null_mut(),
                    0,
                    &mut required
                ),
                -1
            );
            assert_eq!(
                du_legacy_text(
                    1,
                    ptr::null(),
                    1,
                    ptr::null(),
                    0,
                    0,
                    0,
                    ptr::null_mut(),
                    0,
                    &mut required
                ),
                -1
            );
        }
    }
    #[test]
    fn overlap_padding_dequote_and_digits() {
        assert_eq!(
            apply(4, &ascii("aaaa"), &ascii("aa"), 0, 0).unwrap(),
            ascii("3")
        );
        assert_eq!(
            apply(5, &ascii("aaaa"), &ascii("aa"), 0, 0).unwrap(),
            ascii("")
        );
        assert_eq!(apply(1, &[], &[], -1, 32), Ok(vec![]));
        assert_eq!(
            apply(3, &ascii("abcdef"), &[], 3, 32).unwrap(),
            ascii("abc")
        );
        assert_eq!(
            apply(12, &ascii("'a\\'b'"), &[], 92, 39).unwrap(),
            ascii("a\\'b")
        );
        assert_eq!(
            apply(13, &ascii("\"a\\\"b\"tail"), &[], 0, 0).unwrap(),
            ascii("a\\\"b")
        );
        assert_eq!(
            apply(14, &ascii("\"a\\\"b\"tail"), &[], 0, 0).unwrap(),
            ascii("tail")
        );
        assert_eq!(apply(31, &ascii("abc"), &[], 0, 0).unwrap(), ascii("0"));
        assert_eq!(apply(31, &[0xb2], &[], 0, 0), Err(-2));
        assert_eq!(
            apply(34, &ascii("000.1.2.255"), &[], 0, 0).unwrap(),
            ascii("1")
        );
        assert_eq!(
            apply(34, &ascii("+0.1.2.3"), &[], 0, 0).unwrap(),
            ascii("0")
        );
    }
    #[test]
    fn bytes_hex_cipher_and_replacement() {
        unsafe {
            assert_eq!(
                apply(36, &[0, 255, 65], &[], 0, 0).unwrap(),
                ascii("00FF41")
            );
            assert_eq!(apply(37, &ascii("+F-1"), &[], 0, 0), Err(-2));
            assert_eq!(apply(39, &ascii("+F-1"), &[], 0, 0).unwrap(), vec![]);
            assert_eq!(apply(37, &ascii("F"), &[], 0, 0), Err(-2));
            let bytes = [0, 65, 255, 0xc3, 0xa7];
            let encrypted = apply(38, &bytes, &[], 1234, 0).unwrap();
            assert_eq!(apply(39, &encrypted, &[], 1234, 0).unwrap(), bytes);
            assert_eq!(apply(39, &ascii("invalid"), &[], 0, 0).unwrap(), vec![]);
            let source = ascii("aaaa");
            let old = ascii("aa");
            let replacement = [0xd800, 0];
            let mut output = [9u16; 6];
            let mut needed = 0;
            assert_eq!(
                du_legacy_replace(
                    source.as_ptr(),
                    4,
                    old.as_ptr(),
                    2,
                    replacement.as_ptr(),
                    2,
                    output.as_mut_ptr(),
                    3,
                    &mut needed
                ),
                -1
            );
            assert_eq!(output, [9; 6]);
            assert_eq!(needed, 4);
            assert_eq!(
                du_legacy_replace(
                    source.as_ptr(),
                    4,
                    old.as_ptr(),
                    2,
                    replacement.as_ptr(),
                    2,
                    output.as_mut_ptr(),
                    4,
                    &mut needed
                ),
                0
            );
            assert_eq!(output, [0xd800, 0, 0xd800, 0, 9, 9]);
        }
    }
    #[test]
    fn unicode_table_and_accent_are_exact() {
        assert!(number(0xb2));
        assert!(letter(0x41));
        assert!(!number(0xd800));
        assert_eq!(
            apply(22, &[0xb2, 0, 0xd800, 49], &[], 0, 0).unwrap(),
            vec![0xb2, 49]
        );
        assert_eq!(
            apply(6, &[0xd7, 0xf7, 0xd8, 0xff, 0xd800], &[], 0, 0).unwrap(),
            vec![0xd7, 0xf7, 79, 121, 0xd800]
        );
        assert_eq!(
            apply(17, &[0xb0, 0xa6, 39, 124], &[], 0, 0).unwrap(),
            vec![0xa6]
        );
        assert_eq!(apply(27, &[], &[], 0, 0).unwrap(), ascii("1"));
        assert_eq!(apply(25, &[], &[], 0, 0).unwrap(), ascii("0"));
    }
}

/// Case-sensitive, non-overlapping replacement of UTF-16 code units.
#[no_mangle]
pub unsafe extern "C" fn du_legacy_replace(
    a: *const u16,
    alen: usize,
    old: *const u16,
    oldlen: usize,
    replacement: *const u16,
    replacementlen: usize,
    out: *mut u16,
    capacity: usize,
    required: *mut usize,
) -> i32 {
    if required.is_null() || capacity > isize::MAX as usize / 2 || (out.is_null() && capacity != 0)
    {
        return -1;
    }
    for (p, len) in [(a, alen), (old, oldlen), (replacement, replacementlen)].iter() {
        if (*p).is_null() && *len != 0 || *len > isize::MAX as usize / 2 {
            return -1;
        }
    }
    *required = 0;
    let result = std::panic::catch_unwind(|| {
        let units = |p, len| {
            if len == 0 {
                &[][..]
            } else {
                slice::from_raw_parts(p, len)
            }
        };
        let a = units(a, alen);
        let old = units(old, oldlen);
        let replacement = units(replacement, replacementlen);
        let mut result = Vec::new();
        let mut i = 0;
        while i < a.len() {
            if !old.is_empty() && a[i..].starts_with(old) {
                result.extend_from_slice(replacement);
                i += old.len();
            } else {
                result.push(a[i]);
                i += 1;
            }
        }
        result
    });
    match result {
        Ok(value) => {
            *required = value.len();
            if out.is_null() {
                return 0;
            }
            if capacity < value.len() {
                return -1;
            }
            if !value.is_empty() {
                ptr::copy(value.as_ptr(), out, value.len());
            }
            0
        }
        Err(_) => -3,
    }
}
