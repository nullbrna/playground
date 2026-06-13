// Compute the LPS (Longest Prefix Suffix) array for a given pattern.
//
// Find the pattern with both a prefix and a suffix, storing the length of
// full/partial matches. The length of the calculated LPS array is equal to the
// length of the input pattern.

#[allow(unused)]
fn find_all_matches_lengths(pattern: &[u8]) -> Vec<usize> {
    let pattern_length = pattern.len();
    let mut lps = vec![0; pattern_length];

    let mut pattern_index = 1;
    let mut prev_lps_length = 0;

    while pattern_index < pattern_length {
        // Characters match. Extend the current prefix/suffix length.
        if pattern[pattern_index] == pattern[prev_lps_length] {
            prev_lps_length += 1;
            lps[pattern_index] = prev_lps_length;

            pattern_index += 1;
            continue;
        }

        // Don't iterate. Try the previous stored length.
        if prev_lps_length != 0 {
            prev_lps_length = lps[prev_lps_length - 1];
            continue;
        }

        pattern_index += 1;
    }

    lps
}

#[allow(unused)]
fn find_pattern_start_index(source: &str, pattern: &str) -> Option<usize> {
    if source.is_empty() {
        return None;
    } else if pattern.is_empty() {
        return Some(0);
    }

    let source = source.as_bytes();
    let pattern = pattern.as_bytes();

    let lps = find_all_matches_lengths(pattern);

    let mut source_index = 0;
    let mut pattern_index = 0;

    while source_index < source.len() {
        if source[source_index] == pattern[pattern_index] {
            source_index += 1;
            pattern_index += 1;

            // Success state. Full pattern matched. Return the difference
            // between our end position of the match and the pattern length.
            if pattern_index == pattern.len() {
                return Some(source_index - pattern_index);
            }

            continue;
        }

        // Mismatch part-way through a pattern match. Use the calculated LPS to
        // skip wasted pattern comparisons.
        if pattern_index != 0 {
            pattern_index = lps[pattern_index - 1];
            continue;
        }

        source_index += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::find_all_matches_lengths;
    use crate::find_pattern_start_index;

    #[test]
    fn should_return_empty_vec_for_empty_pattern() {
        let pattern = "".as_bytes();
        let lps = find_all_matches_lengths(pattern);

        assert_eq!(lps, vec![]);
    }

    #[test]
    fn should_return_no_length_for_unique_pattern() {
        let pattern = "ABCDE".as_bytes();
        let lps = find_all_matches_lengths(pattern);

        assert_eq!(lps, vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn should_return_growing_length_for_all_repeats() {
        let pattern = "AAAAA".as_bytes();
        let lps = find_all_matches_lengths(pattern);

        assert_eq!(lps, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn should_return_length_for_repeat() {
        let pattern = "ABABC".as_bytes();
        let lps = find_all_matches_lengths(pattern);

        assert_eq!(lps, vec![0, 0, 1, 2, 0]);
    }

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
