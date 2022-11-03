use byte_string::*;

#[test]
fn test_byte_str_size() {
    assert_eq!(std::mem::size_of::<&[u8]>(), 16);
    assert_eq!(std::mem::size_of::<ByteStr>(), 16);
}

mod instantiation {
    use super::*;

    #[test]
    fn test_byte_str_new() {
        let a = ByteStr::new(b"hello");
        let b = ByteStr::new("hello");
        assert_eq!(a, b);
    }

    #[test]
    fn test_byte_string_new() {
        let a = ByteString::new();
        assert_eq!(a.len(), 0);
    }
}

mod copying_and_cloning {
    use super::*;

    #[test]
    fn test_byte_str_copy() {
        let a: ByteStr = "a".into();
        let b: ByteStr = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_byte_string_clone() {
        let a: ByteString = "a".into();
        let b = a.clone();
        assert_eq!(a, b);
    }
}

mod conversions_from {
    use super::*;

    #[test]
    fn test_byte_str_from() {
        let a = ByteStr::from(b"hello");
        let b = ByteStr::from("hello");
        assert_eq!(a, b);
    }

    #[test]
    fn test_byte_string_from() {
        // Vecs and Strings are moved
        let _a = ByteString::from(b"hello".to_vec());
        let _b = ByteString::from("hello".to_string());

        // Slice/reference types are cloned
        let _c = ByteString::from(b"hello");
        let _d = ByteString::from("hello");
    }
}

mod display {
    use super::*;

    #[test]
    fn test_byte_str_display() {
        use std::fmt::Write;

        let a = ByteStr::from("hello");
        let mut buf = String::new();

        write!(buf, "{}", a).unwrap();

        assert_eq!(buf, "hello")
    }

    #[test]
    fn test_byte_string_display() {
        use std::fmt::Write;

        let a = ByteString::from("hello");
        let mut buf = String::new();

        write!(buf, "{}", a).unwrap();

        assert_eq!(buf, "hello")
    }
}

mod deref {
    use super::*;
    #[test]
    fn test_byte_str_deref() {
        let a = ByteStr::from(b"hello");
        assert_eq!(a.len(), 5);
    }

    #[test]
    fn test_byte_string_deref() {
        let a = ByteString::from(b"hello");
        assert_eq!(a.len(), 5);
    }

    #[test]
    fn test_byte_string_mut_deref() {
        let mut a = ByteString::from(b"hello");
        a.push(b'a');
        assert_eq!(a.len(), 6);
    }
}

mod conversions_to {
    use super::*;

    #[test]
    fn test_byte_str_to_str_lossy() {
        let a = ByteStr::from("hello");
        assert_eq!(a.to_str_lossy(), "hello");
    }

    #[test]
    fn test_byte_string_as_byte_str() {
        let a = ByteString::from(b"hello");
        let _b: ByteStr = a.as_byte_str();
    }

    #[test]
    fn test_byte_string_into_vec() {
        fn assert_vec(_: Vec<u8>) {}

        let a = ByteString::from(b"hello");
        assert_vec(a.into_vec());
    }

    #[test]
    fn test_byte_string_as_ref_u8() {
        fn assert_as_ref(_: impl AsRef<[u8]>) {}

        let a = ByteString::from(b"hello");
        assert_as_ref(&a);
        assert_eq!(a.as_ref(), b"hello");
    }

    #[test]
    fn test_byte_str_parse() {
        assert_eq!(ByteStr::from(b"1").parse::<i64>(), Ok(1i64));
        assert_eq!(ByteStr::from(b"x").parse::<i64>(), Err(ParseIntError));
    }

    #[test]
    fn test_byte_string_parse() {
        assert_eq!(ByteString::from(b"1").parse::<i64>(), Ok(1i64));
        assert_eq!(ByteString::from(b"x").parse::<i64>(), Err(ParseIntError));
    }
}

mod ascii_compat {
    use super::*;

    #[test]
    fn test_byte_str_eq_ignore_ascii_case() {
        let a = ByteStr::from("hEllo");
        let b = ByteStr::from("helLo");

        assert!(a.eq_ignore_ascii_case(&b));
        assert!(a.eq_ignore_ascii_case(b"HeLlO"));
        assert!(a.eq_ignore_ascii_case("HeLlO"));
    }

    #[test]
    fn test_byte_str_to_lowercase() {
        let a: ByteStr = "abcABC123\x01".into();
        let b = a.to_lowercase();
        assert_eq!(b, "abcabc123\x01".into())
    }

    #[test]
    fn test_byte_string_to_lowercase() {
        let a: ByteString = "abcABC123\x01".into();
        let b = a.to_lowercase();
        assert_eq!(b, "abcabc123\x01".into())
    }

    #[test]
    fn test_byte_str_to_uppercase() {
        let a: ByteStr = "abcABC123\x01".into();
        let b = a.to_uppercase();
        assert_eq!(b, "ABCABC123\x01".into())
    }

    #[test]
    fn test_byte_string_to_uppercase() {
        let a: ByteString = "abcABC123\x01".into();
        let b = a.to_uppercase();
        assert_eq!(b, "ABCABC123\x01".into())
    }
}

mod in_collections {
    use super::*;

    #[test]
    fn test_byte_string_as_hashmap_key() {
        use std::collections::HashMap;
        let mut h = HashMap::new();
        h.insert(ByteString::from("a"), 1);
    }
}
