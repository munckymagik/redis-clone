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
