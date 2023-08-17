#![warn(clippy::pedantic)]
#![feature(async_fn_in_trait)]

use axum::{extract::DefaultBodyLimit, routing::get, Router};
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

mod storage;
use storage::{InMemoryStore, S3Store};
mod backend;
use backend::Backend;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let _storage = Arc::new(RwLock::new(S3Store::new(
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
    )));
    let storage = Arc::new(RwLock::new(InMemoryStore::new()));

    let app = Router::new()
        .route(
            "/ac/*path",
            get(Backend::get_action).put(Backend::put_action),
        )
        .route("/cas/*path", get(Backend::get_item).put(Backend::put_item))
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
