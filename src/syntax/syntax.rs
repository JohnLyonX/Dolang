// Syntax validation and helper functions for Dolang.

/// ContainsRawEnglish no longer rejects bare ASCII letters (plain identifiers are now allowed).
pub fn contains_raw_english(_line: &str) -> bool {
    false
}

/// All valid $ prefixes in Dolang, ordered longest first to avoid
/// prefix shadowing (e.g. "$continue" must come before "$con").
const DOLLAR_PREFIXES: &[&str] = &[
    "$>>",
    "$continue",
    "$while",
    "$break",
    "$elif",
    "$else",
    "$loop",
    "$for",
    "$fn",
    "$if",
    "$@",
    "$#",
    "$",   // bare $ = variable declaration (must be last)
];

/// ValidateDollarLiterals ensures every `$` in the line is the start of
/// a known Dolang operator or keyword.
pub fn validate_dollar_literals(line: &str) -> (bool, String) {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '$' {
            i += 1;
            continue;
        }

        // Build the suffix starting at `$` for prefix matching
        let suffix: String = chars[i..].iter().collect();

        let mut matched = false;
        for prefix in DOLLAR_PREFIXES {
            if suffix.starts_with(prefix) {
                i += prefix.chars().count();
                matched = true;
                break;
            }
        }

        if !matched {
            // Should never happen since bare "$" is in the list, but be safe
            i += 1;
        }
    }
    (true, String::new())
}
