use bytes::Bytes;
use s3::Bucket;

use super::Storage;

#[derive(Debug)]
pub struct Store {
    bucket: Bucket,
}

impl Store {
    pub fn new(bucket: Bucket) -> Self {
        Self { bucket }
    }
}

impl Storage for Store {
    async fn clear(&mut self) {
        unimplemented!("S3Storage::clear")
    }

    async fn get(&self, key: &str) -> Option<Bytes> {
        self.bucket
            .get_object(key)
            .await
            .ok()
            .map(|resp| resp.to_vec().into())
    }

    async fn has(&self, key: &str) -> bool {
        self.bucket.head_object(key).await.is_ok()
    }

    async fn list(&self, _key: &str) -> Vec<String> {
        unimplemented!("S3Storage::list")
    }

    async fn remove(&mut self, _key: &str) {
        unimplemented!("S3Storage::remove")
    }

    async fn set(&mut self, key: &str, value: Bytes) {
        self.bucket.put_object(key, value.as_ref()).await.unwrap();
    }
}
