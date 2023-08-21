use std::pin::Pin;
use anyhow::Result;

use bytes::Bytes;
use futures::stream::Stream;

mod in_memory;
pub use in_memory::Store as InMemoryStore;

mod s3;
pub use s3::Store as S3Store;

pub trait Storage {
    async fn get(&self, key: &str) -> Option<Pin<Box<dyn Stream<Item=Result<Bytes>> + Send>>>;
    async fn set(&mut self, key: &str, value: Pin<Box<dyn Stream<Item=Result<Bytes>> + Send>>);
    async fn remove(&mut self, key: &str);
    async fn list(&self, key: &str) -> Vec<String>;
    async fn clear(&mut self);
    async fn has(&self, key: &str) -> bool {
        self.get(key).await.is_some()
    }
}
