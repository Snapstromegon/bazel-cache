#![warn(clippy::pedantic)]

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, put},
    Router,
};
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

mod storage;
use storage::{InMemoryStore, S3Store, Storage};
mod backend;
use backend::Backend;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let storage = create_storage(StorageType::InMemory);

    let app = Router::new()
        .route("/ac/*path", get(Backend::get_action))
        .route("/ac/*path", put(Backend::put_action))
        .route("/cas/*path", get(Backend::get_item))
        .route("/cas/*path", put(Backend::put_item))
        .with_state(storage.clone())
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(Backend::shutdown_signal())
        .await
        .unwrap();
}

#[derive(Debug, Clone, Copy)]
enum StorageType {
    S3,
    InMemory,
}

fn create_storage(storage_type: StorageType) -> Arc<RwLock<dyn Storage + Send + Sync>> {
    match storage_type {
        StorageType::InMemory => Arc::new(RwLock::new(InMemoryStore::new())),
        StorageType::S3 => Arc::new(RwLock::new(S3Store::new(
            Bucket::new(
                "test",
                Region::Custom {
                    region: String::new(),
                    endpoint: "http://localhost:9000".to_owned(),
                },
                Credentials::new(Some("minioadmin"), Some("minioadmin"), None, None, None).unwrap(),
            )
            .unwrap()
            .with_path_style(),
        ))),
    }
}
