//! Independent implementation of Delphi 13 Base64's permissive byte contract.
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn encode(input: &[u16], width: i32, separator: &[u16]) -> Result<Vec<u16>, i32> {
    let mut output = Vec::new();
    let mut blocks = 0;
    for chunk in input.chunks(3) {
        if chunk.iter().any(|c| *c > 255) {
            return Err(-2);
        }
        let bits = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        output.push(u16::from(ALPHABET[((bits >> 18) & 63) as usize]));
        output.push(u16::from(ALPHABET[((bits >> 12) & 63) as usize]));
        output.push(if chunk.len() > 1 {
            u16::from(ALPHABET[((bits >> 6) & 63) as usize])
        } else {
            61
        });
        output.push(if chunk.len() > 2 {
            u16::from(ALPHABET[(bits & 63) as usize])
        } else {
            61
        });
        if chunk.len() == 3 {
            blocks += 1;
            if width > 0 && blocks >= width / 4 {
                output.extend_from_slice(separator);
                blocks = 0;
            }
        }
    }
    Ok(output)
}

pub fn decode(input: &[u16]) -> Vec<u16> {
    let mut output = Vec::new();
    let mut bits = 0u32;
    let mut count = 0;
    for unit in input {
        let byte = *unit as u8;
        if let Some(value) = ALPHABET.iter().position(|c| *c == byte) {
            bits = (bits << 6) | value as u32;
            count += 6;
            if count >= 8 {
                count -= 8;
                output.push(((bits >> count) & 255) as u16);
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    fn units(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }
    #[test]
    fn line_breaks_padding_and_all_bytes() {
        assert_eq!(
            encode(&units("foo"), 0, &units("\r\n")).unwrap(),
            units("Zm9v")
        );
        assert_eq!(
            encode(&units("f"), 4, &units("\r\n")).unwrap(),
            units("Zg==")
        );
        assert_eq!(
            encode(&units("foo"), 4, &units("\r\n")).unwrap(),
            units("Zm9v\r\n")
        );
        let at_boundary = encode(&vec![65; 57], 76, &units("\r\n")).unwrap();
        assert_eq!(at_boundary.len(), 78);
        assert_eq!(&at_boundary[76..], &units("\r\n"));
        let bytes: Vec<u16> = (0..=255).collect();
        assert_eq!(decode(&encode(&bytes, 76, &units("\r\n")).unwrap()), bytes);
        assert_eq!(encode(&[256], 0, &[]), Err(-2));
    }
    #[test]
    fn permissive_decode_ignores_padding_and_uses_low_byte() {
        assert_eq!(decode(&units("Z=g==!\r\n")), units("f"));
        assert_eq!(decode(&units("Zg==Zg==")), vec![102, 6, 96]);
        assert_eq!(decode(&[0x15a, 0x167]), units("f"));
        assert!(decode(&units("A=!!!!")).is_empty());
        assert_eq!(decode(&units("AA")), vec![0]);
        assert_eq!(decode(&units("AAA")), vec![0, 0]);
    }
}
