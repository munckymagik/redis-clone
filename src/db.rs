use std::{
    collections::HashMap,
    collections::VecDeque,
    iter::{FromIterator, IntoIterator},
};
use byte_string::ByteString;

pub type Database = HashMap<String, RObj>;

#[derive(Debug, PartialEq)]
pub enum RObj {
    Int(i64),
    String(ByteString),
    List(VecDeque<ByteString>),
}

impl From<i64> for RObj {
    fn from(other: i64) -> Self {
        Self::Int(other)
    }
}

impl From<ByteString> for RObj {
    fn from(other: ByteString) -> Self {
        match other.parse() {
            Ok(n) => Self::Int(n),
            Err(_) => Self::String(other),
        }
    }
}

impl RObj {
    pub fn new_list_from(other: impl IntoIterator<Item = ByteString>) -> Self {
        RObj::List(VecDeque::from_iter(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_list_from() {
        let expected = RObj::List(VecDeque::from(vec!["x".into()]));
        let actual = RObj::new_list_from(vec!["x".into()]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_from_byte_string() {
        // Non-numbers become Strings
        let o: RObj = ByteString::from("a").into();
        assert_eq!(o, RObj::String(ByteString::from("a")));

        // Numbers become Ints
        let o: RObj = ByteString::from("-123").into();
        assert_eq!(o, RObj::Int(-123_i64));

        // The maximum value of an i64 can be stored as an Int
        let max = format!("{}", std::i64::MAX);
        let o: RObj = ByteString::from(max).into();
        assert_eq!(o, RObj::Int(std::i64::MAX));

        // The minimum value of an i64 can be stored as an Int
        let min = format!("{}", std::i64::MIN);
        let o: RObj = ByteString::from(min).into();
        assert_eq!(o, RObj::Int(std::i64::MIN));

        // Overflowing the maximum value of an i64 results in a String
        let o: RObj = ByteString::from(format!("{}1", std::i64::MAX)).into();
        assert_eq!(o, RObj::String(ByteString::from("92233720368547758071")));
    }
}
