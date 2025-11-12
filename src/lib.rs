//! Encode base58.

/// Encoding alphabet from Bitcoin.
static TABLE: [u8; 58] = *b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

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

/// Append the encoded 32-byte array into an output string.  The result
/// can be trimed as well.
pub fn encode32_append(v: &[u8; 32], output: &mut String, trim: bool) {
    let res = encode32(v);

    // strip the leading zeros
    let mut start = 0;
    while trim && start < 48 && res[start] == b'1' {
        start += 1;
    }

    // Safety: our alphabet is only ASCII chars
    let s = unsafe { core::str::from_utf8_unchecked(&res[start..]) };

    output.push_str(s);
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
    fn test_one_string() {
        let mut output = String::new();
        encode32_append(
            &[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x01,
            ],
            &mut output,
            true,
        );
        assert_eq!("2", output,);
    }
}
