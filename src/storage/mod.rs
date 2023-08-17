pub trait Storage {
    async fn get(&self, key: &str) -> Option<Vec<u8>>;
    async fn set(&mut self, key: &str, value: Vec<u8>);
    async fn remove(&mut self, key: &str);
    async fn list(&self, key: &str) -> Vec<String>;
    async fn clear(&mut self);
    async fn has(&self, key: &str) -> bool {
        self.get(key).await.is_some()
    }
}

mod in_memory;
pub use in_memory::Store as InMemoryStore;

mod s3;
pub use s3::Store as S3Store;
