use std::borrow::Cow;
use std::fmt::{self, Display};

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

#[derive(Debug, Default, PartialEq)]
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
}

impl From<Vec<u8>> for ByteString
{
    fn from(other: Vec<u8>) -> Self {
        ByteString { bytes: other }
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
        let _a = ByteString::from(b"hello".to_vec());
        let _b = ByteString::from(b"hello");
        let _c = ByteString::from("hello");
        let _d: ByteString = "hello".into();
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
}
