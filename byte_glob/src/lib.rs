//! Port of the stringmatchlen glob style string matching algorithm from Redis.
//! Works with byte slices rather than strict utf-8 strings, so it can be used
//! to match binary data.

pub fn glob(pattern: &[u8], string: &[u8]) -> bool {
    let mut pattern = &pattern[..];
    let mut string = &string[..];

    while !pattern.is_empty() && !string.is_empty() {
        match pattern[0] {
            b'*' => {
                // Skip over repeated asterisks
                while let Some(b'*') = pattern.get(1) {
                    pattern = &pattern[1..];
                }

                if pattern.len() == 1 {
                    // Last char of the pattern so we can assume a match till
                    // the end
                    return true;
                }

                // Search ahead through the string
                while !string.is_empty() {
                    // Recursive search with whatever pattern comes after the
                    // asterisk. Recursing is necessary so we can use
                    // combinations of wildcards and escapes next to each
                    // other.
                    if glob(&pattern[1..], string) {
                        // If the rest of the pattern matches we're done
                        return true;
                    }

                    // No match, consume another byte from the string and try
                    // again
                    string = &string[1..];
                }
            }
            b'?' => string = &string[1..],
            b'[' => {
                // Advance to the first range character
                pattern = &pattern[1..];

                // Catch the case where the opening brace is the last character
                // of the match and has not been escaped
                if pattern.is_empty() {
                    return false;
                }

                let mut found = false;
                let not = pattern[0] == b'^';

                if not {
                    // "Not" flag detected, advance to the next character
                    pattern = &pattern[1..];
                }

                loop {
                    // Defend against unexpectedly exhausted pattern. E.g if
                    // there is a stray '-' at the end of the range expression.
                    if pattern.is_empty() {
                        break;
                    }

                    match pattern[0] {
                        // We must test the escape before the closing square
                        // bracket so we can escape a closing square
                        // bracket
                        b'\\' => {
                            // Advance the pattern to the char
                            // being escaped
                            pattern = &pattern[1..];

                            // Do a literal match
                            if pattern[0] == string[0] {
                                found = true;
                            }
                        }
                        // We found the natural end of the range, stop matching
                        b']' => break,
                        _ => {
                            if pattern.len() >= 3 && pattern[1] == b'-' {
                                let start = pattern[0];
                                let end = pattern[2];
                                let mut range = start..=end;

                                if start > end {
                                    range = (*range.end())..=(*range.start())
                                }

                                if range.contains(&string[0]) {
                                    found = true;
                                }

                                // Step to the 'end' char of the range
                                // expression
                                pattern = &pattern[2..];
                            } else if pattern[0] == string[0] {
                                found = true;
                            }
                        }
                    }

                    // We must walk the entire range until the closing square
                    // bracket, advance to next
                    pattern = &pattern[1..];
                }

                if not {
                    found = !found;
                }

                if !found {
                    return false;
                }

                // We found a match, advance the string
                string = &string[1..];
            }
            _ => {
                if pattern[0] == b'\\' && pattern.len() > 1 {
                    // The current pattern char is the escape and this is not
                    // the last char in the pattern, so we advance to the next
                    // match character and and will treat it as a literal match
                    pattern = &pattern[1..];
                }

                if pattern[0] != string[0] {
                    return false;
                }

                string = &string[1..];
            }
        }

        // Defend against unexpectedly exhausted pattern.
        if pattern.is_empty() {
            break;
        }

        // Advance the pattern
        pattern = &pattern[1..];

        // If the string is already exhausted we'll need to break the loop
        if string.is_empty() {
            // If the remaining pattern chars are asterisks we can skip them,
            // so there's still a chance of a match
            while let Some(b'*') = pattern.get(0) {
                pattern = &pattern[1..]
            }

            break;
        }
    }

    if pattern.is_empty() && string.is_empty() {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_inputs() {
        assert!(glob(b"", b""));
        assert!(!glob(b"a", b""));
        assert!(!glob(b"", b"a"));
    }

    #[test]
    fn literal_characters() {
        assert!(glob(b"a", b"a"));
        assert!(!glob(b"a", b"b"));
    }

    #[test]
    fn literal_characters_different_lengths() {
        // When the pattern ends before the string
        assert!(!glob(b"a", b"aa"));

        // When the string ends before the pattern
        assert!(!glob(b"aa", b"a"));
    }

    #[test]
    fn literal_characters_same_length() {
        assert!(glob(b"aa", b"aa"));
        assert!(!glob(b"aa", b"ab"));
    }

    #[test]
    fn wildcard_question_mark() {
        assert!(glob(b"?", b"a"));
        assert!(!glob(b"?", b""));
        assert!(glob(b"?a", b"aa"));
        assert!(glob(b"a?", b"aa"));
        assert!(glob(b"??", b"aa"));
    }

    #[test]
    fn wildcard_asterisk() {
        // Matches anything
        assert!(glob(b"*", b"a"));
        assert!(glob(b"*", b"\0\x01abcdefABCDEF12345;'.,*?"));

        // But not an empty string
        assert!(!glob(b"*", b""));

        // When trailing matches till the end
        assert!(glob(b"*", b"ab"));
        assert!(glob(b"a*", b"ab"));
        assert!(glob(b"a*", b"abc"));
        assert!(!glob(b"b*", b"abc"));

        // When leading searches for next match
        assert!(glob(b"*c", b"abc"));
        assert!(!glob(b"*d", b"abc"));
        assert!(!glob(b"*c", b"abcd"));

        // When in the centre it searches for next match
        assert!(glob(b"a*c", b"abc"));
        assert!(!glob(b"a*d", b"abc"));
        assert!(!glob(b"a*c", b"abcd"));

        // Treats contiguous asterisks as one
        assert!(glob(b"a**c", b"abc"));
        assert!(glob(b"a***c", b"abc"));
        assert!(glob(b"a**", b"abc"));
        assert!(glob(b"**c", b"abc"));

        // Skips trailing asterisks if the string has already
        // been exhausted
        assert!(glob(b"abc*", b"abc"));
        assert!(glob(b"abc**", b"abc"));
    }

    #[test]
    fn escapes() {
        // When there are only escapes
        assert!(glob(br#"\"#, br#"\"#));
        assert!(glob(br#"\\"#, br#"\"#));
        assert!(!glob(br#"\\\"#, br#"\"#));
        assert!(glob(br#"\\\"#, br#"\\"#));
        assert!(glob(br#"\\\\"#, br#"\\"#));

        // It forces a literal match of the next character
        assert!(glob(br#"\a"#, br#"a"#));
        assert!(!glob(br#"\\"#, br#"a"#));
        assert!(!glob(br#"\a"#, br#"b"#));

        // When preceding a wildcard it forces a literal match
        assert!(glob(br#"\*"#, br#"*"#));
        assert!(!glob(br#"\*"#, br#"a"#));
        assert!(glob(br#"\?"#, br#"?"#));
        assert!(!glob(br#"\?"#, br#"a"#));
    }

    #[test]
    fn range_match() {
        // Empty range never matches
        assert!(!glob(b"[]", b""));
        assert!(!glob(b"[]", b"[]"));

        // When escaped matches literal square brackets
        assert!(glob(b"\\[]", b"[]"));

        // Allows matching the closing bracket when escaped
        assert!(glob(b"[\\]]", b"]"));

        // Matches any one of the range of bytes
        assert!(glob(b"[aA1;\0]", b"a"));
        assert!(glob(b"[aA1;\0]", b"A"));
        assert!(glob(b"[aA1;\0]", b"1"));
        assert!(glob(b"[aA1;\0]", b";"));
        assert!(glob(b"[aA1;\0]", b"\0"));

        // Not matches
        assert!(!glob(b"[^a]", b"a"));
        assert!(glob(b"[^a]", b"b"));

        // Dash range
        assert!(!glob(b"[1-3]", b"0"));
        assert!(glob(b"[1-3]", b"1"));
        assert!(glob(b"[1-3]", b"2"));
        assert!(glob(b"[1-3]", b"3"));
        assert!(!glob(b"[1-3]", b"4"));

        // Dash at start will match a literal dash
        assert!(glob(b"[-]", b"-"));
        assert!(glob(b"[-3]", b"-"));
        assert!(glob(b"[-3]", b"3"));

        // Dash at the end won't match
        assert!(!glob(b"[3-]", b"-"));
        assert!(glob(b"[3-]", b"3"));

        // When there is no closing delimeter,
        assert!(!glob(b"[", b"["));
        assert!(glob(b"\\[", b"["));
        // weirdly these work even on real Redis
        assert!(glob(b"[123\\]", b"2"));
        assert!(glob(b"[123", b"2"));
        assert!(!glob(b"[123", b"4"));
        assert!(glob(b"[1-3", b"2"));

        // Reverse range
        assert!(glob(b"[3-1]", b"2"));
    }

    #[test]
    fn permutations() {
        // Asterisks with question marks
        assert!(glob(b"*?", b"ab"));
        assert!(glob(b"*?", b"abc"));
        assert!(glob(b"*?c", b"abc"));
        assert!(glob(b"?*", b"a"));
        assert!(glob(b"?*", b"ab"));
        assert!(glob(b"??*", b"ab"));
        assert!(glob(b"??*", b"abc"));

        // Asterisks with ranges
        assert!(glob(b"*[*]", b"a*"));
        assert!(glob(b"*[b]", b"ab"));
        assert!(glob(b"*[c]", b"abc"));
        assert!(glob(b"[a]*", b"a"));
        assert!(glob(b"[a]*", b"ab"));
        assert!(!glob(b"[b]*", b"ab"));

        // Asterisks with escapes
        assert!(glob(b"*\\*", b"a*"));
        assert!(glob(b"\\**", b"*a"));

        // Question marks with ranges
        assert!(glob(b"?[?]", b"a?"));
        assert!(glob(b"[?]?", b"?a"));

        // Question marks with escapes
        assert!(glob(b"?\\?", b"a?"));
        assert!(glob(b"\\??", b"?a"));

        // Ranges with escapes
        assert!(glob(b"[\\]]\\[", b"]["));
        assert!(glob(b"\\[[\\]]", b"[]"));

        // Random examples
        assert!(glob(b"abc*\\[", b"abc*["));
    }
}
