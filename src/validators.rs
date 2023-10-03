use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Validators {
    inner: Arc<Mutex<HashMap<String, u32>>>,
}

impl Validators {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn insert(&self, key: &str, value: u32) {
        let mut validators = self.inner.lock().unwrap();

        let enrty = validators.entry(key.to_string()).or_insert(0);
        *enrty = value;
    }

    pub fn len(&self) -> usize {
        let validators = self.inner.lock().unwrap();

        validators.len()
    }

    pub fn stake(&self, key: &str) -> Option<u32> {
        let validators = self.inner.lock().unwrap();
        validators.get(key).cloned()
    }
}
