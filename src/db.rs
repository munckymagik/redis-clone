use std::{
    collections::HashMap,
    collections::VecDeque,
    iter::{FromIterator, IntoIterator},
};

pub type Database = HashMap<String, RObj>;

#[derive(Debug, PartialEq)]
pub enum RObj {
    String(String),
    List(VecDeque<String>),
}

impl From<String> for RObj {
    fn from(other: String) -> Self {
        Self::String(other)
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
}
