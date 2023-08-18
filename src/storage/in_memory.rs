use bytes::Bytes;
use std::{collections::HashMap, io::Cursor, pin::Pin};
use tokio::io::AsyncRead;

use super::Storage;

#[derive(Debug)]
pub struct Store {
    data: HashMap<String, Bytes>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Storage for Store {
    async fn get(&self, key: &str) -> Option<Pin<Box<dyn AsyncRead>>> {
        Some(Box::pin(Cursor::new(self.data.get(key)?.clone())))
    }

    async fn set(&mut self, key: &str, value: Bytes) {
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
