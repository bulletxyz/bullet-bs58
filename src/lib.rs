//! Encode base58.

/// Encoding alphabet from Bitcoin.
const TABLE: [u8; 58] = *b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const REVERSE: [u8; 256] = {
    let mut i = 0;
    let mut res = [255u8; 256];
    while i < TABLE.len() {
        res[TABLE[i] as usize] = i as u8;
        i += 1;
    }
    res
};

/// Encode a fixed 32-byte array into a 48 byte array.
pub fn encode32(v: &[u8; 32]) -> [u8; 48] {
    let mut num: [u64; 4] =
        core::array::from_fn(|i| u64::from_be_bytes(v[i * 8..i * 8 + 8].try_into().unwrap()));

    // init all elements with zero to enable early out.
    let mut res = [b'1'; 48];

    for output in res.chunks_mut(8).rev() {
        // extract eight output characters at once
        const PART: u128 = 128063081718016; // 58**8
        let mut rem = num[0] % PART as u64;
        num[0] /= PART as u64;
        for elem in num.iter_mut().skip(1) {
            let x = (*elem as u128) + ((rem as u128) << 64);
            rem = (x % PART) as u64;
            *elem = (x / PART) as u64;
        }

        // the compiler is not clever enough to unroll this code - do it by hand
        output[7] = TABLE[(rem % 58) as usize];
        output[6] = TABLE[(rem / 58 % 58) as usize];
        output[5] = TABLE[(rem / (58 * 58) % 58) as usize];
        output[4] = TABLE[(rem / (58 * 58 * 58) % 58) as usize];
        rem /= 58 * 58 * 58 * 58;
        if rem != 0 {
            output[3] = TABLE[(rem % 58) as usize];
            output[2] = TABLE[(rem / 58 % 58) as usize];
            output[1] = TABLE[(rem / (58 * 58) % 58) as usize];
            output[0] = TABLE[(rem / (58 * 58 * 58) % 58) as usize];
        }
    }
    res
}

/// Append the encoded 32-byte array into an output string. This is
/// the canonical encoding with as much leading `1` digits as zeros
/// are in the input.
pub fn encode32_append(v: &[u8; 32], output: &mut String) {
    let res = encode32(v);

    // strip all ones
    let mut start = 0;
    for ch in res {
        if ch != b'1' {
            break;
        }
        start += 1;
    }

    // add as many back as there are zeros in the input
    for ch in v {
        if *ch != 0 || start == 0 {
            break;
        }
        start -= 1;
    }

    // Safety: our alphabet is only ASCII chars
    let s = unsafe { core::str::from_utf8_unchecked(&res[start..]) };

    output.push_str(s);
}

/// Parse a base58 string into an 32-byte array.
pub fn parse32(input: &[u8]) -> Option<[u8; 32]> {
    let mut num = [0u64; 4];
    for ch in input {
        let mut carry = REVERSE[*ch as usize] as u64;
        if carry >= 58 {
            // invalid char
            return None;
        }
        for p in num.iter_mut() {
            let v = (*p as u128) * 58 + carry as u128;
            *p = v as u64;
            carry = (v >> 64) as u64;
        }
        if carry != 0 {
            // overflow 32-bytes
            return None;
        }
    }
    let res: [_; 4] = core::array::from_fn(|i| u64::to_be_bytes(num[3 - i]));
    Some(unsafe { core::mem::transmute::<[[u8; 8]; 4], [u8; 32]>(res) })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_zero() {
        assert_eq!([b'1'; 48], encode32(&[0; 32]));
    }
    #[test]
    fn test_one() {
        assert_eq!(
            *b"111111111111111111111111111111111111111111111112",
            encode32(&[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x01,
            ])
        );
    }
    #[test]
    fn test_large() {
        assert_eq!(
            *b"11119cfBkPsoQ2NPHYPi7b69bcQG8FKfNc33k2UfRxiPFyd9",
            encode32(&[
                0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ])
        );
    }
    #[test]
    fn test_complex() {
        assert_eq!(
            *b"11112gPihUTjt3FJqf1VpidgrY5cZ6PuyMccGVwQHRfjMPZG",
            encode32(&[
                0x18, 0xf3, 0x06, 0xdf, 0xe6, 0x99, 0xd2, 0x08, 0x5c, 0x89, 0x7b, 0x43, 0xa4, 0xc5,
                0x4f, 0xc4, 0x7d, 0x2b, 0xb7, 0x55, 0x67, 0x5b, 0xe8, 0xa7, 0x49, 0x83, 0x68, 0x83,
                0x00, 0x65, 0xd6, 0xe7
            ])
        );
    }
    #[test]
    fn test_parse_overflow() {
        assert_eq!(None, parse32(&[b'z'; 44]));
    }
    #[test]
    fn test_append_matches_bs58() {
        let cases = [
            // 38 chars
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            // 39 chars
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0,
            ],
            // 40 chars
            [
                0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            // 41 chars
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 252, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0,
            ],
            // 42 chars
            [
                0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            // 43 chars
            [
                0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0,
            ],
            [0u8; 32],
            {
                let mut x = [0u8; 32];
                x[31] = 1;
                x
            },
            {
                let mut x = [0u8; 32];
                x[0] = 1;
                x
            },
            {
                let mut x = [0u8; 32];
                x[0] = 0xff;
                x
            },
            [0xff; 32],
            [
                0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
                0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x13, 0x57, 0x9b, 0xdf,
                0x24, 0x68, 0xac, 0xe0,
            ],
        ];

        for bytes in cases {
            let mut left = String::new();
            encode32_append(&bytes, &mut left);
            let right = bs58::encode(bytes).into_string();
            assert_eq!(left, right, "{} vs {}", left.len(), right.len());
        }
    }
}
