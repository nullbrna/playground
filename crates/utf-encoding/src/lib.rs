// UTF-8 (Unicode Transformation Format)
//
// 0xxxxxxx - Single ASCII character (fits in 128 bits. MSB is zero)
// 110xxxxx 10xxxxxx - 2-byte character (latin-1) e.g. ñ
// 1110xxxx 10xxxxxx 10xxxxxx - 3-byte character e.g. €
// 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx - 4-byte character e.g. 😀
//
// The payload (code point) of each UTF-8 character can be classified as the
// non-metadata bits i.e. all BUT the leading length and continuation bits.

#[allow(unused)]
fn raw_length(source: &str) -> usize {
    let bytes = source.as_bytes();

    let mut count = 0;
    let mut index = 0;

    // Using the leading byte, determine the total count of UTF-8 bytes.
    while let Some(byte) = bytes.get(index) {
        // Restrict ambiguous checks by validating the extra bit is explicitly
        // turned off e.g. 2-byte strings get a mask of 11100000.
        let length = match byte {
            value if value & 0b1000_0000 == 0b0000_0000 => 1,
            value if value & 0b1110_0000 == 0b1100_0000 => 2,
            value if value & 0b1111_0000 == 0b1110_0000 => 3,
            value if value & 0b1111_1000 == 0b1111_0000 => 4,
            _ => return 0,
        };

        count += length;
        index += length;
    }

    count
}

#[allow(unused)]
fn is_valid(source: &str) -> bool {
    let bytes = source.as_bytes();

    let mut index = 0;
    while let Some(leading_byte) = bytes.get(index) {
        let length = match leading_byte {
            value if value & 0b1000_0000 == 0b0000_0000 => 1,
            value if value & 0b1110_0000 == 0b1100_0000 => 2,
            value if value & 0b1111_0000 == 0b1110_0000 => 3,
            value if value & 0b1111_1000 == 0b1111_0000 => 4,
            _ => return false,
        };

        // Ensure the remaining is prefixed with 10 i.e. are continuation bytes.
        if let Some(rest) = bytes.get(index + 1..index + length)
            && rest.iter().all(|byte| byte & 0b1100_0000 == 0b1000_0000)
        {
            index += length;
            continue;
        }

        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use crate::is_valid;
    use crate::raw_length;

    #[test]
    fn utf_zero_bytes_length() {
        assert_eq!(raw_length(""), 0);
    }

    #[test]
    fn utf_one_byte_chars_length() {
        assert_eq!(raw_length("foo"), 3);
    }

    #[test]
    fn utf_two_byte_chars_length() {
        assert_eq!(raw_length("ñññ"), 6);
    }

    #[test]
    fn utf_three_byte_chars_length() {
        assert_eq!(raw_length("€€"), 6);
    }

    #[test]
    fn utf_four_byte_chars_length() {
        assert_eq!(raw_length("😀"), 4);
    }

    #[test]
    fn utf_zero_bytes_valid() {
        assert_eq!(is_valid(""), true);
    }

    #[test]
    fn utf_one_byte_chars_valid() {
        assert_eq!(is_valid("foo"), true);
    }

    #[test]
    fn utf_two_byte_chars_valid() {
        assert_eq!(is_valid("ñññ"), true);
    }
}
