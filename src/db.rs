use std::collections::HashMap;

pub type Database = HashMap<String, RObj>;

pub enum RObj {
    String(String),
    List(Vec<String>),
}

impl From<String> for RObj {
    fn from(other: String) -> Self {
        Self::String(other)
    }
}

impl From<Vec<String>> for RObj {
    fn from(other: Vec<String>) -> Self {
        Self::List(other)
    }
}
