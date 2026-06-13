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
fn raw_byte_length(source: &str) -> usize {
    let bytes = source.as_bytes();

    let mut count = 0;
    let mut index = 0;

    while let Some(byte) = bytes.get(index) {
        // NOTE: Restrict ambiguous checks by validating the extra bit is
        // explicitly turned off e.g. 2-byte strings get a mask of 11100000.
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
fn has_correct_byte_order(source: &str) -> bool {
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
        // Failing this means the length is wrong or the bytes are malformed.
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
    use crate::has_correct_byte_order;
    use crate::raw_byte_length;

    #[test]
    fn should_return_zero_for_empty_input() {
        assert_eq!(raw_byte_length(""), 0);
    }

    #[test]
    fn should_return_equal_to_characters_length() {
        assert_eq!(raw_byte_length("foo"), 3);
    }

    #[test]
    fn should_return_double_characters_length() {
        assert_eq!(raw_byte_length("ñññ"), 6);
    }

    #[test]
    fn should_return_triple_characters_length() {
        assert_eq!(raw_byte_length("€€"), 6);
    }

    #[test]
    fn should_return_quadruple_characters_length() {
        assert_eq!(raw_byte_length("😀"), 4);
    }

    #[test]
    fn should_return_true_for_empty_input() {
        assert_eq!(has_correct_byte_order(""), true);
    }

    #[test]
    fn should_return_true_for_single_byte_characters() {
        assert_eq!(has_correct_byte_order("foo"), true);
    }

    #[test]
    fn should_return_true_for_double_byte_characters() {
        assert_eq!(has_correct_byte_order("ñññ"), true);
    }
}
