// Find the first occurrence of a pattern within a string using "Boyer Moore".
//
// Compare characters right to left, using the "Bad Character" rule. Mismatches
// allow for the iterator to skip forward based on the last occurrence of said
// mismatch within a pattern.

#[allow(unused)]
fn find_pattern_start_index(source: &str, pattern: &str) -> Option<usize> {
    let source = source.as_bytes();
    let pattern = pattern.as_bytes();

    let source_length = source.len();
    let pattern_length = pattern.len();

    if source.is_empty() || pattern_length > source_length {
        return None;
    } else if pattern.is_empty() {
        return Some(0);
    }

    // Store the last index where each byte appears in the pattern. Used if a
    // mismatch occurs to shift the pattern forward. Characters not present
    // remain -1.
    let mut bad_char_table = [-1; 256];
    for idx in 0..pattern_length {
        bad_char_table[pattern[idx] as usize] = idx as i32;
    }

    let mut skipped = 0;
    while skipped <= source_length - pattern_length {
        let mut offset = (pattern_length - 1) as i32;

        // Iterate backwards through the pattern while the characters match. Map
        // the pattern index to the current source index.
        while offset >= 0 && pattern[offset as usize] == source[skipped + offset as usize] {
            offset -= 1;
        }

        if offset < 0 {
            return Some(skipped);
        }

        let last_occurrence = bad_char_table[source[skipped + offset as usize] as usize];
        skipped += (offset - last_occurrence).max(1) as usize;
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::find_pattern_start_index;

    #[test]
    fn should_return_none_for_empty_input() {
        let source = "";
        let pattern = "";

        assert_eq!(find_pattern_start_index(source, pattern), None);
    }

    #[test]
    fn should_return_start_of_source() {
        let source = "hello, world!";
        let pattern = "hel";

        assert_eq!(find_pattern_start_index(source, pattern), Some(0));
    }

    #[test]
    fn should_return_middle_of_source() {
        let source = "hello, world!";
        let pattern = ", w";

        assert_eq!(find_pattern_start_index(source, pattern), Some(5));
    }

    #[test]
    fn should_return_end_of_source() {
        let source = "hello, world!";
        let pattern = "ld!";

        assert_eq!(find_pattern_start_index(source, pattern), Some(10));
    }

    #[test]
    fn should_return_first_match() {
        let source = "worldhelloworldhello";
        let pattern = "hello";

        assert_eq!(find_pattern_start_index(source, pattern), Some(5));
    }

    #[test]
    fn should_return_none_for_no_match() {
        let source = "hello, world!";
        let pattern = "foo";

        assert_eq!(find_pattern_start_index(source, pattern), None);
    }

    #[test]
    fn should_return_none_for_longer_pattern() {
        let source = "hello";
        let pattern = "hello, world!";

        assert_eq!(find_pattern_start_index(source, pattern), None);
    }
}
