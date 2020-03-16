use byte_string::ByteString;
use std::{
    collections::hash_map::Keys,
    collections::HashMap,
    collections::VecDeque,
    iter::{FromIterator, IntoIterator},
    time::Instant,
};

pub struct Database {
    inner: HashMap<ByteString, RObj>,
    expires: HashMap<ByteString, Instant>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            expires: HashMap::new(),
        }
    }

    pub fn get<'a>(&'a mut self, key: &ByteString) -> Option<&'a RObj> {
        if self.remove_if_expired(key) {
            return None;
        }

        self.inner.get(key)
    }

    pub fn get_mut<'a>(&'a mut self, key: &ByteString) -> Option<&'a mut RObj> {
        if self.remove_if_expired(key) {
            return None;
        }

        self.inner.get_mut(key)
    }

    pub fn keys(&mut self) -> Keys<'_, ByteString, RObj> {
        self.inner.keys()
    }

    pub fn clear(&mut self) {
        // Clears all the key-values but retains memory
        self.inner.clear();
        self.expires.clear();

        // Releases memory
        self.inner.shrink_to_fit();
        self.expires.shrink_to_fit();
    }

    pub fn insert(&mut self, key: ByteString, value: RObj) {
        self.inner.insert(key, value);
    }

    pub fn remove(&mut self, key: &ByteString) -> Option<RObj> {
        self.expires.remove(key);
        self.inner.remove(key)
    }

    pub fn set_expire(&mut self, key: &ByteString, expires_at: Instant) -> bool {
        if !self.inner.contains_key(key) {
            return false;
        };

        self.expires.insert(key.clone(), expires_at);

        true
    }

    pub fn get_expire(&self, key: &ByteString) -> Option<Instant> {
        self.expires.get(key).copied()
    }

    pub fn persist(&mut self, key: &ByteString) -> bool {
        self.expires.remove(key).is_some()
    }

    fn is_expired(&self, key: &ByteString) -> bool {
        match self.get_expire(key) {
            Some(when) => Instant::now() > when,
            _ => false,
        }
    }

    fn remove_if_expired(&mut self, key: &ByteString) -> bool {
        if self.is_expired(key) {
            self.remove(key);
            return true;
        }

        false
    }
}

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
    use std::time::Duration;

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

    #[test]
    fn test_expires() {
        let now = Instant::now();
        let mut db = Database::new();
        let key: ByteString = "x".into();
        let expires_at = now + Duration::from_secs(10);

        // When there is no associated key
        assert_eq!(db.get_expire(&key), None);
        assert!(!db.set_expire(&key, expires_at));
        assert!(!db.persist(&key));

        // When there is an associated key but no expiry
        db.insert(key.clone(), 123.into());
        assert_eq!(db.get_expire(&key), None);
        assert!(!db.persist(&key));

        // When there is an associated key and an expiry
        assert!(db.set_expire(&key, expires_at));
        assert_eq!(db.get_expire(&key), Some(expires_at));
        assert!(db.persist(&key));
        assert_eq!(db.get_expire(&key), None);
    }

    #[test]
    fn test_clear() {
        let mut db = Database::new();
        let now = Instant::now();
        let key: ByteString = "x".into();
        let expires_at = now + Duration::from_secs(10);
        db.insert(key.clone(), 123.into());
        db.set_expire(&key, expires_at);

        // Before clearing
        assert_eq!(db.inner.len(), 1);
        assert_eq!(db.expires.len(), 1);

        // After clearing
        db.clear();
        assert_eq!(db.inner.len(), 0);
        assert_eq!(db.expires.len(), 0);
        assert_eq!(db.inner.capacity(), 0);
        assert_eq!(db.expires.capacity(), 0);
    }

    #[test]
    fn test_is_expired() {
        let mut db = Database::new();
        let key: ByteString = "x".into();
        db.insert(key.clone(), 123.into());

        // When there is no expiry set
        {
            // It should be reported as NOT expired
            assert!(!db.is_expired(&key));
        }

        // When there is an expiry in the past
        {
            let expires_at = Instant::now() - Duration::from_secs(10);
            db.set_expire(&key, expires_at);

            // It should be reported as expired
            assert!(db.is_expired(&key));
        }

        // When there is an expiry in the future
        {
            let expires_at = Instant::now() + Duration::from_secs(10);
            db.set_expire(&key, expires_at);

            // It should be reported as NOT expired
            assert!(!db.is_expired(&key));
        }
    }

    #[test]
    fn test_remove() {
        let mut db = Database::new();
        let key: ByteString = "x".into();

        // When there is no expiry set
        {
            db.insert(key.clone(), 123.into());

            // It should remove the key
            assert!(db.remove(&key).is_some());
            assert!(!db.inner.contains_key(&key));
        }

        // When there is an expiry set
        {
            db.insert(key.clone(), 123.into());

            let expires_at = Instant::now() + Duration::from_secs(10);
            db.set_expire(&key, expires_at);

            // It should remove the key
            assert!(db.remove(&key).is_some());
            assert!(!db.inner.contains_key(&key));

            // It should also remove the expiry
            assert!(db.get_expire(&key).is_none());
        }
    }

    #[test]
    fn test_remove_if_expired() {
        let mut db = Database::new();
        let key: ByteString = "x".into();
        db.insert(key.clone(), 123.into());

        // When there is no expiry set
        {
            // It should not be removed
            assert!(!db.remove_if_expired(&key));
            assert!(db.inner.contains_key(&key));
        }

        // When there is an expiry in the future
        {
            let expires_at = Instant::now() + Duration::from_secs(10);
            db.set_expire(&key, expires_at);

            // It should not be removed
            assert!(!db.remove_if_expired(&key));
            assert!(db.inner.contains_key(&key));
        }

        // When there is an expiry in the past
        {
            let expires_at = Instant::now() - Duration::from_millis(1);
            db.set_expire(&key, expires_at);

            // It should be removed
            assert!(db.remove_if_expired(&key));
            assert!(!db.inner.contains_key(&key));
        }
    }

    #[test]
    fn test_get() {
        let mut db = Database::new();
        let key: ByteString = "x".into();
        db.insert(key.clone(), 123.into());

        // When there is no expiry set
        {
            // It should return the value
            assert!(db.get(&key).is_some());
            assert!(db.inner.contains_key(&key));
        }

        // When there is an expiry in the future
        {
            let expires_at = Instant::now() + Duration::from_secs(10);
            db.set_expire(&key, expires_at);

            // It should return the value
            assert!(db.get(&key).is_some());
            assert!(db.inner.contains_key(&key));
        }

        // When there is an expiry in the past
        {
            let expires_at = Instant::now() - Duration::from_millis(1);
            db.set_expire(&key, expires_at);

            // It should return none
            assert!(db.get(&key).is_none());

            // It should have removed the value
            assert!(!db.inner.contains_key(&key));
        }
    }

    #[test]
    fn test_get_mut() {
        let mut db = Database::new();
        let key: ByteString = "x".into();
        db.insert(key.clone(), 123.into());

        // When there is no expiry set
        {
            // It should return the value
            assert!(db.get_mut(&key).is_some());
            assert!(db.inner.contains_key(&key));
        }

        // When there is an expiry in the future
        {
            let expires_at = Instant::now() + Duration::from_secs(10);
            db.set_expire(&key, expires_at);

            // It should return the value
            assert!(db.get_mut(&key).is_some());
            assert!(db.inner.contains_key(&key));
        }

        // When there is an expiry in the past
        {
            let expires_at = Instant::now() - Duration::from_millis(1);
            db.set_expire(&key, expires_at);

            // It should return none
            assert!(db.get_mut(&key).is_none());

            // It should have removed the value
            assert!(!db.inner.contains_key(&key));
        }
    }
}
