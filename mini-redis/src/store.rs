use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Entry {
    value: String,
    expires_at: Option<Instant>,
}

impl Entry {
    fn is_expired(&self) -> bool {
        matches!(self.expires_at, Some(t) if Instant::now() >= t)
    }
}

#[derive(Clone)]
pub struct Store {
    data: Arc<Mutex<HashMap<String, Entry>>>,
}

/// Lazy expiry - if the key exists but its time is up we should delete it now.
fn remove_if_expired(data: &mut HashMap<String, Entry>, key: &str) {
    if data.get(key).is_some_and(|e| e.is_expired()) {
        data.remove(key);
    }
}

impl Store {
    pub fn new() -> Self {
        Store { data: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn set(&self, key: String, value: String, ttl_secs: Option<u64>) {
        let expires_at = ttl_secs.map(|s| Instant::now() + Duration::from_secs(s));
        let mut data = self.data.lock().unwrap();
        data.insert(key, Entry { value, expires_at });
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let mut data = self.data.lock().unwrap();
        remove_if_expired(&mut data, key);
        data.get(key).map(|e| e.value.clone())
    }

    pub fn del(&self, key: &str) -> bool {
        let mut data = self.data.lock().unwrap();
        remove_if_expired(&mut data, key);
        data.remove(key).is_some()
    }

    pub fn exists(&self, key: &str) -> bool {
        let mut data = self.data.lock().unwrap();
        remove_if_expired(&mut data, key);
        data.contains_key(key)
    }

    /// -2 = no such key, -1 = no expiry, otherwise seconds remaining.
    pub fn ttl(&self, key: &str) -> i64 {
        let mut data = self.data.lock().unwrap();
        remove_if_expired(&mut data, key);
        match data.get(key) {
            None => -2,
            Some(entry) => match entry.expires_at {
                None => -1,
                Some(t) => t
                    .saturating_duration_since(Instant::now())
                    .as_secs_f64()
                    .ceil() as i64,
            },
        }
    }

    pub fn expire(&self, key: &str, seconds: u64) -> bool {
        let mut data = self.data.lock().unwrap();
        remove_if_expired(&mut data, key);
        match data.get_mut(key) {
            Some(entry) => {
                entry.expires_at = Some(Instant::now() + Duration::from_secs(seconds));
                true
            }
            None => false,
        }
    }

    /// Remove every expired key & Returns how many were removed
    pub fn purge_expired(&self) -> usize {
        let mut data = self.data.lock().unwrap();
        let before = data.len();
        data.retain(|_, entry| !entry.is_expired());
        before - data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_then_get() {
        let s = Store::new();
        s.set("a".into(), "1".into(), None);
        assert_eq!(s.get("a"), Some("1".to_string()));
        assert_eq!(s.get("missing"), None);
    }

    #[test]
    fn del_reports_whether_key_existed() {
        let s = Store::new();
        s.set("a".into(), "1".into(), None);
        assert!(s.del("a"));
        assert!(!s.del("a"));
    }

    #[test]
    fn expired_key_is_gone() {
        let s = Store::new();
        s.set("a".into(), "1".into(), Some(0)); // expires immediately
        assert_eq!(s.get("a"), None);
        assert!(!s.exists("a"));
    }

    #[test]
    fn ttl_codes() {
        let s = Store::new();
        assert_eq!(s.ttl("missing"), -2);
        s.set("a".into(), "1".into(), None);
        assert_eq!(s.ttl("a"), -1);
        s.expire("a", 100);
        assert!((99..=100).contains(&s.ttl("a")));
    }

    #[test]
    fn purge_removes_only_expired() {
        let s = Store::new();
        s.set("old".into(), "1".into(), Some(0));
        s.set("keep".into(), "2".into(), None);
        assert_eq!(s.purge_expired(), 1);
        assert_eq!(s.get("keep"), Some("2".to_string()));
    }
}