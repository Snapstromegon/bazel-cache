#![warn(clippy::pedantic)]
#![feature(async_fn_in_trait)]

use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use std::{net::SocketAddr, sync::Arc};
use tokio::{signal, sync::RwLock};

mod storage;
use storage::{Storage, S3Store, InMemoryStore};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let _storage = create_s3_storage();
    let storage = Arc::new(RwLock::new(InMemoryStore::new()));

    // build our application with a route
    let app = Router::new()
        .route("/ac/*path", get(get_action).put(put_action))
        .route("/cas/*path", get(get_item).put(put_item))
        .with_state(storage)
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("shutting down");
}

fn create_s3_storage() -> Arc<RwLock<S3Store>> {
    let mut bucket = Bucket::new(
        "test",
        Region::Custom {
            region: String::new(),
            endpoint: "http://localhost:9000".to_owned(),
        },
        Credentials::new(Some("minioadmin"), Some("minioadmin"), None, None, None).unwrap(),
    )
    .unwrap();
    bucket.set_path_style();
    Arc::new(RwLock::new(S3Store::new(bucket)))
}

async fn get_action<Store: Storage + std::fmt::Debug>(
    Path(path): Path<String>,
    State(cache): State<Arc<RwLock<Store>>>,
) -> impl IntoResponse {
    if let Some(data) = cache.read().await.get(&format!("ac/{path}")).await {
        tracing::info!("Responding from cache for ac/{}", path);
        Ok(data)
    } else {
        tracing::info!("ac/{} not in cache", path);
        Err(StatusCode::NOT_FOUND)
    }
}
async fn put_action<Store: Storage + std::marker::Sync>(
    Path(path): Path<String>,
    State(storage): State<Arc<RwLock<Store>>>,
    body: Bytes,
) -> impl IntoResponse {
    if storage.read().await.has(&format!("ac/{path}")).await {
        StatusCode::CREATED
    } else {
        tracing::info!("Storing in cache for ac/{}", path);
        storage.write().await.set(&format!("ac/{path}"), body.to_vec()).await;
        StatusCode::CREATED
    }
}
async fn get_item<Store: Storage>(
    Path(path): Path<String>,
    State(storage): State<Arc<RwLock<Store>>>,
) -> impl IntoResponse {
    if let Some(data) = storage.read().await.get(&format!("cas/{path}")).await {
        tracing::info!("Responding from cache for cas/{}", path);
        Ok(data)
    } else {
        tracing::info!("cas/{} not in cache", path);
        Err((StatusCode::NOT_FOUND, "not found"))
    }
}
async fn put_item<Store: Storage + std::marker::Sync>(
    Path(path): Path<String>,
    State(storage): State<Arc<RwLock<Store>>>,
    body: Bytes,
) -> impl IntoResponse {
    tracing::info!("Storing in cache for cas/{}", path);
    if storage.read().await.has(&format!("ac/{path}")).await {
        StatusCode::CREATED
    } else {
        storage
            .write()
            .await
            .set(&format!("cas/{path}"), body.to_vec())
            .await;
        StatusCode::CREATED
    }
}
