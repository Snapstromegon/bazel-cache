use bytes::{Bytes, BytesMut};
use std::{collections::HashMap, pin::Pin};
use futures::stream::{Stream, StreamExt};
use async_stream::stream;
use anyhow::Result;

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
    async fn get(&self, key: &str) -> Option<Pin<Box<dyn Stream<Item=Result<Bytes>> + Send>>> {
        let value = self.data.get(key)?.clone();
        Some(Box::pin(stream! {
            yield Ok(value);
        }))
    }

    async fn set(&mut self, key: &str, value: Pin<Box<dyn Stream<Item=Result<Bytes>>>>) {
        let mut result = match value.size_hint() {
            (_, Some(size)) => BytesMut::with_capacity(size),
            _ => BytesMut::new()
        };

        let mut stream = value;
        
        while let Some(chunk) = stream.next().await {
            if let Ok(data) = chunk {
                result.extend_from_slice(&data);
            }
            else {
                panic!("Failed to upload Data")
            }
        }
        self.data.insert(key.to_string(), result.freeze());
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
