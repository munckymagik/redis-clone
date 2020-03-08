use std::borrow::Cow;

mod from_bytes;
use from_bytes::from_bytes;
pub use from_bytes::{Number, ParseIntError};

#[derive(Clone, Copy, Debug, PartialEq)]
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

    pub fn parse<T: Number>(&self) -> Result<T, ParseIntError> {
        from_bytes(self)
    }

    pub fn to_lowercase(&self) -> ByteString {
        let lowered_bytes = self.bytes
            .iter()
            .map(u8::to_ascii_lowercase)
            .collect::<Vec<u8>>();

        ByteString::from(lowered_bytes)
    }

    pub fn to_uppercase(&self) -> ByteString {
        let uppered_bytes = self.bytes
            .iter()
            .map(u8::to_ascii_uppercase)
            .collect::<Vec<u8>>();

        ByteString::from(uppered_bytes)
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
        self.as_byte_str().to_lowercase()
    }

    pub fn to_uppercase(&self) -> Self {
        self.as_byte_str().to_uppercase()
    }
}

mod impl_from {
    use super::*;

    impl<'outer, T> From<&'outer T> for ByteStr<'outer>
    where
        T: AsRef<[u8]> + ?Sized
    {
        fn from(other: &'outer T) -> Self {
            ByteStr { bytes: other.as_ref() }
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
}

mod impl_as_ref {
    use super::*;

    impl AsRef<[u8]> for ByteStr<'_> {
        fn as_ref(&self) -> &[u8] {
            self.bytes
        }
    }

    impl AsRef<[u8]> for ByteString {
        fn as_ref(&self) -> &[u8] {
            self.bytes.as_ref()
        }
    }
}

mod impl_display {
    use super::*;
    use std::fmt::{self, Display};

    impl Display for ByteStr<'_> {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            write!(f, "{}", self.to_str_lossy())
        }
    }

    impl Display for ByteString {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            self.as_byte_str().fmt(f)
        }
    }
}

mod impl_deref {
    use super::*;
    use std::ops::{Deref, DerefMut};

    impl<'a> Deref for ByteStr<'a> {
        type Target = &'a [u8];

        fn deref(&self) -> &Self::Target {
            &self.bytes
        }
    }

    impl Deref for ByteString {
        type Target = Vec<u8>;

        fn deref(&self) -> &Self::Target {
            &self.bytes
        }
    }

    impl DerefMut for ByteString {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.bytes
        }
    }
}
