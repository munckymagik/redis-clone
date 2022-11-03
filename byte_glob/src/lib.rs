#![forbid(unsafe_code)]
#![no_std]

//! Port of the stringmatchlen glob style string matching algorithm from Redis.
//! Works with byte slices rather than strict utf-8 strings, so it can be used
//! to match binary data.

macro_rules! handle_asterisk {
    ($pattern:ident, $string:ident) => {{
        // Skip over repeated asterisks
        while let Some(b'*') = $pattern.get(1) {
            $pattern = &$pattern[1..];
        }

        if $pattern.len() == 1 {
            // Last char of the pattern so we can assume a match till
            // the end
            return true;
        }

        // Search ahead through the string
        while !$string.is_empty() {
            // Recursive search with whatever pattern comes after the
            // asterisk. Recursing is necessary so we can use
            // combinations of wildcards and escapes next to each
            // other.
            if glob(&$pattern[1..], $string) {
                // If the rest of the pattern matches we're done
                return true;
            }

            // No match, consume another byte from the string and try
            // again
            $string = &$string[1..];
        }
    }};
}

#[inline(always)]
fn byte_range(start: u8, end: u8) -> core::ops::RangeInclusive<u8> {
    let mut range = start..=end;

    if start > end {
        range = (*range.end())..=(*range.start())
    }

    range
}

macro_rules! handle_range {
    ($pattern:ident, $string:ident) => {{
        let mut found = false;

        // Advance to the first range character
        $pattern = &$pattern[1..];

        // Detect the "not" flag
        let not = match $pattern.get(0) {
            Some(b'^') => {
                // Advance to the next character
                $pattern = &$pattern[1..];
                true
            }
            _ => false,
        };

        let string_head: u8 = *$string.get(0).expect("string was exhausted");

        loop {
            match $pattern.get(0) {
                // We must test the escape before the closing square
                // bracket so we can escape a closing square
                // bracket
                Some(b'\\') => {
                    // Advance the pattern to the char
                    // being escaped
                    $pattern = &$pattern[1..];

                    // Do a literal match
                    match $pattern.get(0) {
                        Some(&a) if a == string_head => found = true,
                        Some(_) => (),

                        // When the pattern is is exhausted
                        _ => return false,
                    }
                }
                // We found the natural end of the range, stop matching
                Some(b']') => break,
                Some(&pattern_head) => {
                    if $pattern.len() >= 3 && $pattern[1] == b'-' {
                        let range = byte_range(pattern_head, $pattern[2]);
                        if range.contains(&string_head) {
                            found = true;
                        }

                        // Step to the 'end' char of the range
                        // expression
                        $pattern = &$pattern[2..];
                    } else if pattern_head == string_head {
                        found = true;
                    }
                }
                // Defend against unexpectedly exhausted pattern. E.g if
                // there is a stray '-' at the end of the range expression.
                None => break,
            }

            // We must walk the entire range until the closing square
            // bracket, advance to next
            $pattern = &$pattern[1..];
        } // end loop

        if not {
            found = !found;
        }

        if !found {
            return false;
        }

        // We found a match, advance the string
        $string = &$string[1..];
    }};
}

macro_rules! handle_literal {
    ($pattern:ident, $string:ident) => {{
        if $pattern[0] == b'\\' && $pattern.len() > 1 {
            // The current pattern char is the escape and this is not
            // the last char in the pattern, so we advance to the next
            // match character and and will treat it as a literal match
            $pattern = &$pattern[1..];
        }

        if $pattern[0] != $string[0] {
            return false;
        }

        $string = &$string[1..];
    }};
}

pub fn glob(mut pattern: &[u8], mut string: &[u8]) -> bool {
    while !pattern.is_empty() && !string.is_empty() {
        match pattern[0] {
            b'*' => handle_asterisk!(pattern, string),
            b'?' => string = &string[1..],
            b'[' => handle_range!(pattern, string),
            _ => handle_literal!(pattern, string),
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
            while let Some(b'*') = pattern.first() {
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
