use std::{
    collections::HashMap,
    collections::VecDeque,
    iter::{FromIterator, IntoIterator},
};

pub type Database = HashMap<String, RObj>;

#[derive(Debug, PartialEq)]
pub enum RObj {
    Int(i64),
    String(String),
    List(VecDeque<String>),
}

impl From<i64> for RObj {
    fn from(other: i64) -> Self {
        Self::Int(other)
    }
}

impl From<String> for RObj {
    fn from(other: String) -> Self {
        match other.parse::<i64>() {
            Ok(n) => Self::Int(n),
            Err(_) => Self::String(other),
        }
    }
}

impl RObj {
    pub fn new_list_from(other: impl IntoIterator<Item = String>) -> Self {
        RObj::List(VecDeque::from_iter(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_list_from() {
        let expected = RObj::List(VecDeque::from(vec!["x".to_string()]));
        let actual = RObj::new_list_from(vec!["x".to_string()]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_from_string() {
        let o: RObj = "a".to_owned().into();
        assert_eq!(o, RObj::String("a".to_owned()));

        let o: RObj = "1".to_owned().into();
        assert_eq!(o, RObj::Int(1_i64));

        let o: RObj = "-1".to_owned().into();
        assert_eq!(o, RObj::Int(-1_i64));

        let o: RObj = format!("{}", std::i64::MAX).into();
        assert_eq!(o, RObj::Int(std::i64::MAX));

        let o: RObj = format!("{}1", std::i64::MAX).into();
        assert_eq!(o, RObj::String("92233720368547758071".to_owned()));
    }
}
