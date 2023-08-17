use std::collections::HashMap;

use super::Storage;

#[derive(Debug)]
pub struct Store {
    data: HashMap<String, Vec<u8>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Storage for Store {
    async fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    async fn set(&mut self, key: &str, value: Vec<u8>) {
        self.data.insert(key.to_string(), value);
    }
    async fn clear(&mut self) {
        self.data.clear();
    }
    async fn remove(&mut self, key: &str) {
        self.data.remove(key);
    }
    async fn list(&self, key: &str) -> Vec<String> {
        self.data
            .keys()
            .filter(|k| k.starts_with(key))
            .map(ToString::to_string)
            .collect()
    }
    async fn has(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}
