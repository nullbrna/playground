// UTF-8 (Unicode Transformation Format)
//
// 0xxxxxxx - Single ASCII character (fits in 128 bits; MSB is zero)
// 110xxxxx 10xxxxxx - 2-byte character (latin-1) e.g. ñ
// 1110xxxx 10xxxxxx 10xxxxxx - 3-byte character e.g. €
// 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx - 4-byte character e.g. 😀
//
// The payload (code point) of each UTF-8 character can be classified as the
// non-metadata bits i.e. all BUT the leading length and continuation bits.

/// Using each characters leading byte, determine the total count of UTF-8
/// bytes.
/// NOTE: Rust validates a string reference as UTF-8 at compile-time so this is
/// technically redundant.
#[allow(unused)]
fn raw_length(source: &str) -> usize {
    let bytes = source.as_bytes();

    let mut count = 0;
    let mut index = 0;

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

/// For non-single byte UTF-8 strings, check the following continuation bytes
/// for a valid format.
#[allow(unused)]
fn is_valid(source: &str) -> bool {
    let bytes = source.as_bytes();

    let mut index = 0;
    while let Some(byte) = bytes.get(index) {
        let length = match byte {
            value if value & 0b1000_0000 == 0b0000_0000 => 1,
            value if value & 0b1110_0000 == 0b1100_0000 => 2,
            value if value & 0b1111_0000 == 0b1110_0000 => 3,
            value if value & 0b1111_1000 == 0b1111_0000 => 4,
            _ => return false,
        };

        // Using the above length, check the next bytes ensuring they're
        // prefixed with 10.
        for offset in 1..length {
            let check = |source| source & 0b1100_0000 != 0b1000_0000;
            let is_invalid = bytes.get(index + offset).is_some_and(check);

            if is_invalid {
                return false;
            }
        }

        index += length;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

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
