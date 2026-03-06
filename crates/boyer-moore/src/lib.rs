/// Find the first occurrence of a `pattern` within a string using the BM
/// (Boyer-Moore) algorithm.
///
/// Compare characters right-to-left, using the Bad Character rule; mismatches
/// allow for the iterator to skip forward based on the last occurrence of said
/// mismatch within `pattern`.
#[allow(unused)]
fn bm_search(source: &str, pattern: &str) -> Option<usize> {
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
    //
    // TEST
    // T = 3
    // E = 1
    // S = 2
    let mut bad_char_table = [-1; 256];
    for idx in 0..pattern_length {
        bad_char_table[pattern[idx] as usize] = idx as i32;
    }

    // Current offset of the pattern relative to the source.
    let mut skipped = 0;

    while skipped <= source_length - pattern_length {
        // Start comparing from the END of the pattern.
        let mut offset = (pattern_length - 1) as i32;

        // Iterate backwards through the pattern while the characters match. Map
        // the pattern index to the current source index.
        while offset >= 0 && pattern[offset as usize] == source[skipped + offset as usize] {
            offset -= 1;
        }

        // At -1, full pattern match.
        if offset < 0 {
            return Some(skipped);
        }

        // Get the last index of the mismatch within the pattern. Defaults to -1
        // for an unknown character.
        let last_occurrence = bad_char_table[source[skipped + offset as usize] as usize];
        // AAASXT
        //    |
        //    X
        //    |
        // TEXT
        //
        // (3 - -1).max(1) = 4
        skipped += (offset - last_occurrence).max(1) as usize;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bm_search_empty_source_and_pattern() {
        let source = "";
        let pattern = "";

        assert_eq!(bm_search(source, pattern), None);
    }

    #[test]
    fn bm_search_prefix_match() {
        let source = "hello, world!";
        let pattern = "hel";

        assert_eq!(bm_search(source, pattern), Some(0));
    }

    #[test]
    fn bm_search_middle_match() {
        let source = "hello, world!";
        let pattern = ", w";

        assert_eq!(bm_search(source, pattern), Some(5));
    }

    #[test]
    fn bm_search_suffix_match() {
        let source = "hello, world!";
        let pattern = "ld!";

        assert_eq!(bm_search(source, pattern), Some(10));
    }

    #[test]
    fn bm_search_multiple_matches() {
        let source = "worldhelloworldhello";
        let pattern = "hello";

        assert_eq!(bm_search(source, pattern), Some(5));
    }

    #[test]
    fn bm_search_match_not_found() {
        let source = "hello, world!";
        let pattern = "foo";

        assert_eq!(bm_search(source, pattern), None);
    }

    #[test]
    fn bm_search_longer_pattern_than_source() {
        let source = "hello";
        let pattern = "hello, world!";

        assert_eq!(bm_search(source, pattern), None);
    }
}
