use byte_glob::*;

#[test]
fn test_empty_inputs() {
    assert!(glob(b"", b""));
    assert!(!glob(b"a", b""));
    assert!(!glob(b"", b"a"));
}

#[test]
fn test_literal_characters() {
    assert!(glob(b"a", b"a"));
    assert!(!glob(b"a", b"b"));
}

#[test]
fn test_literal_characters_different_lengths() {
    // When the pattern ends before the string
    assert!(!glob(b"a", b"aa"));

    // When the string ends before the pattern
    assert!(!glob(b"aa", b"a"));
}

#[test]
fn test_literal_characters_same_length() {
    assert!(glob(b"aa", b"aa"));
    assert!(!glob(b"aa", b"ab"));
}

#[test]
fn test_wildcard_question_mark() {
    assert!(glob(b"?", b"a"));
    assert!(!glob(b"?", b""));
    assert!(glob(b"?a", b"aa"));
    assert!(glob(b"a?", b"aa"));
    assert!(glob(b"??", b"aa"));
}

#[test]
fn test_wildcard_asterisk() {
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
fn test_escapes() {
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
#[allow(clippy::cognitive_complexity)]
fn test_range_match() {
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
fn test_permutations() {
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
