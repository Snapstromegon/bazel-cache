use std::pin::Pin;

use anyhow::{Context, Result};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use s3::Bucket;
use tokio_util::io::StreamReader;

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

impl Store {
    fn get_blanced_key(&self, key: &str) -> String {
        let parts = key.split("/").collect::<Vec<&str>>();
        let hash = parts[parts.len() - 1];
        let mut balanced_key = String::new();
        for i in 0..parts.len() - 1 {
            balanced_key.push_str(parts[i]);
            balanced_key.push('/');
        }
        balanced_key.push_str(hash[0..2].to_string().as_str());
        balanced_key.push('/');
        balanced_key.push_str(hash);
        balanced_key
    }
}

impl Storage for Store {
    async fn clear(&mut self) {
        unimplemented!("S3Storage::clear")
    }

    async fn get(&self, key: &str) -> Option<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>> {
        if let Ok(stream) = self.bucket.get_object_stream(self.get_blanced_key(key)).await {
            Some(
                stream
                    .bytes
                    .map(|b| b.context("Transforming error"))
                    .boxed(),
            )
        } else {
            None
        }
    }

    async fn has(&self, key: &str) -> bool {
        self.bucket.head_object(self.get_blanced_key(key)).await.is_ok()
    }

    async fn list(&self, _key: &str) -> Vec<String> {
        unimplemented!("S3Storage::list")
    }

    async fn remove(&mut self, _key: &str) {
        unimplemented!("S3Storage::remove")
    }

    async fn set(&mut self, key: &str, value: Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>) {
        let mut reader = StreamReader::new(value.map(|c| {
            if let Ok(c) = c {
                Ok(c)
            } else {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Transforming error"))
            }
        }));
        self.bucket
            .put_object_stream(&mut reader, self.get_blanced_key(key))
            .await
            .unwrap();
    }
}
