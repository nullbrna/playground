/// Compute the LPS (Longest Prefix Suffix) array for a given pattern.
///
/// The length of the returned LPS array is equal to the length of `pattern`.
fn kmp_failure_function(pattern: &str) -> Vec<usize> {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let pattern_length = pattern_chars.len();

    if pattern_length == 0 {
        return Vec::new();
    }

    let mut lps = Vec::with_capacity(pattern_length);
    // First element is always 0. No prefix for a single character.
    lps[0] = 0;

    let mut pattern_index = 1;
    let mut prev_lps_length = 0;

    while pattern_index < pattern_length {
        if pattern_chars[pattern_index] == pattern_chars[prev_lps_length] {
            prev_lps_length += 1;
            lps[pattern_index] = prev_lps_length;
            pattern_index += 1;
            continue;
        }

        if prev_lps_length != 0 {
            prev_lps_length = lps[prev_lps_length - 1];
            continue;
        }

        lps[pattern_index] = 0;
        pattern_index += 1;
    }

    lps
}
