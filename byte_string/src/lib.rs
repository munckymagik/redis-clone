use std::borrow::Cow;
use std::fmt::{self, Display};

mod from_bytes;
use from_bytes::from_bytes;
pub use from_bytes::{Number, ParseIntError};

#[derive(Debug, PartialEq)]
pub struct ByteStr<'inner> {
    bytes: &'inner [u8],
}

impl<'outer> ByteStr<'outer> {
    pub fn new<T>(other: &'outer T) -> Self
    where
        T: AsRef<[u8]> + ?Sized
    {
        ByteStr { bytes: other.as_ref() }
    }

    pub fn to_str_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self.bytes)
    }

    pub fn eq_ignore_ascii_case<T>(&self, other: T) -> bool
    where
        T: AsRef<[u8]>
    {
        self.bytes.eq_ignore_ascii_case(other.as_ref())
    }
}

impl<'outer, T> From<&'outer T> for ByteStr<'outer>
where
    T: AsRef<[u8]> + ?Sized
{
    fn from(other: &'outer T) -> Self {
        ByteStr { bytes: other.as_ref() }
    }
}

impl AsRef<[u8]> for ByteStr<'_> {
    fn as_ref(&self) -> &[u8] {
        self.bytes
    }
}

impl Display for ByteStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_str_lossy())
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct ByteString {
    bytes: Vec<u8>,
}

impl ByteString {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn as_byte_str(&self) -> ByteStr<'_> {
        ByteStr::new(&self.bytes)
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.bytes
    }

    pub fn parse<T: Number>(&self) -> Result<T, ParseIntError> {
        from_bytes(self)
    }

    pub fn to_lowercase(&self) -> Self {
        let lowered_bytes = self.bytes
            .iter()
            .map(u8::to_ascii_lowercase)
            .collect::<Vec<u8>>();

        Self::from(lowered_bytes)
    }
}

impl From<Vec<u8>> for ByteString
{
    fn from(other: Vec<u8>) -> Self {
        ByteString { bytes: other }
    }
}

impl From<String> for ByteString {
    fn from(other: String) -> Self {
        ByteString { bytes: other.into_bytes() }
    }
}

impl<T> From<&T> for ByteString
where
    T: AsRef<[u8]> + ?Sized
{
    fn from(other: &T) -> Self {
        ByteString { bytes: other.as_ref().to_vec() }
    }
}

impl std::ops::Deref for ByteString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl std::ops::DerefMut for ByteString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytes
    }
}

impl AsRef<[u8]> for ByteString {
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

impl Display for ByteString {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.as_byte_str().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_str_size() {
        assert_eq!(std::mem::size_of::<&[u8]>(), 16);
        assert_eq!(std::mem::size_of::<ByteStr>(), 16);
    }
    #[test]
    fn test_byte_str_new() {
        let a = ByteStr::new(b"hello");
        let b = ByteStr::new("hello");
        assert_eq!(a, b);
    }

    #[test]
    fn test_byte_str_from() {
        let a = ByteStr::from(b"hello");
        let b = ByteStr::from("hello");
        assert_eq!(a, b);
    }

    #[test]
    fn test_byte_str_to_str_lossy() {
        let a = ByteStr::from("hello");
        assert_eq!(a.to_str_lossy(), "hello");
    }

    #[test]
    fn test_byte_str_display() {
        use std::fmt::Write;

        let a = ByteStr::from("hello");
        let mut buf = String::new();

        write!(buf, "{}", a).unwrap();

        assert_eq!(buf, "hello")
    }

    #[test]
    fn test_byte_str_eq_ignore_ascii_case() {
        let a = ByteStr::from("hEllo");
        let b = ByteStr::from("helLo");

        assert!(a.eq_ignore_ascii_case(&b));
        assert!(a.eq_ignore_ascii_case(b"HeLlO"));
        assert!(a.eq_ignore_ascii_case("HeLlO"));
    }

    #[test]
    fn test_byte_string() {
        let a = ByteString::new();
        assert_eq!(a.len(), 0);
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

    #[test]
    fn test_byte_string_display() {
        use std::fmt::Write;

        let a = ByteString::from("hello");
        let mut buf = String::new();

        write!(buf, "{}", a).unwrap();

        assert_eq!(buf, "hello")
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

    #[test]
    fn test_byte_string_as_byte_str() {
        let a = ByteString::from(b"hello");
        let _b: ByteStr = a.as_byte_str();
    }

    #[test]
    fn test_byte_string_into_vec() {
        fn assert_vec(_: Vec<u8>) {};

        let a = ByteString::from(b"hello");
        assert_vec(a.into_vec());
    }

    #[test]
    fn test_byte_string_as_ref_u8() {
        fn assert_as_ref(_: impl AsRef<[u8]>) {};

        let a = ByteString::from(b"hello");
        assert_as_ref(&a);
        assert_eq!(a.as_ref(), b"hello");
    }

    #[test]
    fn test_byte_string_as_hashmap_key() {
        use std::collections::HashMap;
        let mut h = HashMap::new();
        h.insert(ByteString::from("a"), 1);
    }

    #[test]
    fn test_byte_string_clone() {
        let a: ByteString = "a".into();
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_byte_string_parse() {
        assert_eq!(ByteString::from(b"1").parse::<i64>(), Ok(1i64));
        assert_eq!(ByteString::from(b"x").parse::<i64>(), Err(ParseIntError));
    }

    #[test]
    fn test_byte_string_to_lowercase() {
        let a: ByteString = "abcABC123\x01".into();
        let b = a.to_lowercase();
        assert_eq!(b, "abcabc123\x01".into())
    }
}
