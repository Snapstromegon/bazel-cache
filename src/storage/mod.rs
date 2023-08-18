use std::pin::Pin;

use bytes::Bytes;
use tokio::io::AsyncRead;

mod in_memory;
pub use in_memory::Store as InMemoryStore;

// mod s3;
// pub use s3::Store as S3Store;

pub trait Storage {
    async fn get(&self, key: &str) -> Option<Pin<Box<dyn AsyncRead>>>;
    async fn set(&mut self, key: &str, value: Bytes);
    async fn remove(&mut self, key: &str);
    async fn list(&self, key: &str) -> Vec<String>;
    async fn clear(&mut self);
    async fn has(&self, key: &str) -> bool {
        self.get(key).await.is_some()
    }
}
