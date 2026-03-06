/// Compute the LPS (Longest Prefix Suffix) array for a given pattern.
///
/// Find the pattern with both a prefix and a suffix, storing the length of
/// full/partial matches. The length of the returned LPS array is equal to the
/// length of `pattern`.
#[allow(unused)]
fn kmp_failure_function(pattern: &[u8]) -> Vec<usize> {
    let pattern_length = pattern.len();
    let mut lps = vec![0; pattern_length];

    // First element is always 0 as there's no proper prefix.
    let mut pattern_index = 1;
    // Count of characters matched as we iterate.
    let mut prev_lps_length = 0;

    while pattern_index < pattern_length {
        // Characters match, extend the current prefix/suffix length.
        if pattern[pattern_index] == pattern[prev_lps_length] {
            prev_lps_length += 1;
            // Increment length to account for this iteration match.
            lps[pattern_index] = prev_lps_length;

            pattern_index += 1;
            continue;
        }

        // Don't iterate, try the previous stored length.
        if prev_lps_length != 0 {
            prev_lps_length = lps[prev_lps_length - 1];
            continue;
        }

        // Mismatch with no previous prefix to fall back to.
        pattern_index += 1;
    }

    lps
}

/// Find the starting index of a matched pattern.
///
/// Use a failure function to iterate over the `source` comparing to the
/// corresponding `pattern` without double-checking segments.
#[allow(unused)]
fn kmp_search(source: &str, pattern: &str) -> Option<usize> {
    if source.is_empty() {
        return None;
    } else if pattern.is_empty() {
        return Some(0);
    }

    let source = source.as_bytes();
    let pattern = pattern.as_bytes();

    let lps = kmp_failure_function(pattern);

    let mut source_index = 0;
    let mut pattern_index = 0;

    while source_index < source.len() {
        // Characters match, increment both indices.
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

        // Mismatch part-way through a pattern match. Use LPS to skip wasted
        // pattern comparisons.
        if pattern_index != 0 {
            pattern_index = lps[pattern_index - 1];
            continue;
        }

        // Mismatch at pattern start so iterate forward.
        source_index += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kmp_failure_function_empty_pattern() {
        let pattern = "".as_bytes();
        let lps = kmp_failure_function(pattern);

        assert_eq!(lps, vec![]);
    }

    #[test]
    fn kmp_failure_function_no_repeats() {
        let pattern = "ABCDE".as_bytes();
        let lps = kmp_failure_function(pattern);

        assert_eq!(lps, vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn kmp_failure_function_all_repeats() {
        let pattern = "AAAAA".as_bytes();
        let lps = kmp_failure_function(pattern);

        assert_eq!(lps, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn kmp_failure_function_partial_repeat() {
        let pattern = "ABABC".as_bytes();
        let lps = kmp_failure_function(pattern);

        assert_eq!(lps, vec![0, 0, 1, 2, 0]);
    }

    #[test]
    fn kmp_search_empty_source_and_pattern() {
        let source = "";
        let pattern = "";

        assert_eq!(kmp_search(source, pattern), None);
    }

    #[test]
    fn kmp_search_prefix_match() {
        let source = "hello, world!";
        let pattern = "hel";

        assert_eq!(kmp_search(source, pattern), Some(0));
    }

    #[test]
    fn kmp_search_middle_match() {
        let source = "hello, world!";
        let pattern = ", w";

        assert_eq!(kmp_search(source, pattern), Some(5));
    }

    #[test]
    fn kmp_search_suffix_match() {
        let source = "hello, world!";
        let pattern = "ld!";

        assert_eq!(kmp_search(source, pattern), Some(10));
    }

    #[test]
    fn kmp_search_match_not_found() {
        let source = "hello, world!";
        let pattern = "foo";

        assert_eq!(kmp_search(source, pattern), None);
    }

    #[test]
    fn kmp_search_longer_pattern_than_source() {
        let source = "hello";
        let pattern = "hello, world!";

        assert_eq!(kmp_search(source, pattern), None);
    }
}
