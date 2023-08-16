use async_trait::async_trait;
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
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{signal, sync::RwLock};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let mut bucket = Bucket::new(
        "test",
        Region::Custom {
            region: "".to_owned(),
            endpoint: "http://localhost:9000".to_owned(),
        },
        Credentials::new(Some("minioadmin"), Some("minioadmin"), None, None, None).unwrap(),
    )
    .unwrap();
    bucket.set_path_style();

    let storage = Arc::new(RwLock::new(S3Storage::new(bucket)));
    // let storage = Arc::new(RwLock::new(InMemoryStorage::new()));

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

async fn get_action<Store: Storage + std::fmt::Debug>(
    Path(path): Path<String>,
    State(cache): State<Arc<RwLock<Store>>>,
) -> impl IntoResponse {
    let cache = cache.read().await;
    if let Some(data) = cache.get(&format!("ac/{path}")).await {
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
    tracing::info!("Storing in cache for ac/{}", path);
    if storage.read().await.has(&format!("ac/{path}")).await {
        StatusCode::CREATED
    } else {
        storage.write().await.set(&format!("ac/{path}"), body).await;
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
            .set(&format!("cas/{path}"), body)
            .await;
        StatusCode::CREATED
    }
}

#[async_trait]
trait Storage {
    async fn get(&self, key: &str) -> Option<Bytes>;
    async fn set(&mut self, key: &str, value: Bytes);
    async fn remove(&mut self, key: &str);
    async fn list(&self, key: &str) -> Vec<String>;
    async fn clear(&mut self);
    async fn has(&self, key: &str) -> bool {
        self.get(key).await.is_some()
    }
}

#[derive(Debug)]
struct S3Storage {
    bucket: Bucket,
}

impl S3Storage {
    fn new(bucket: Bucket) -> Self {
        Self { bucket }
    }
}

#[async_trait]
impl Storage for S3Storage {
    async fn clear(&mut self) {
        unimplemented!("S3Storage::clear")
    }

    async fn get(&self, key: &str) -> Option<Bytes> {
        self.bucket.get_object(key).await.ok().map(|data| data.bytes().clone())
    }

    async fn has(&self, key: &str) -> bool {
        self.bucket.head_object(key).await.is_ok()
    }

    async fn list(&self, key: &str) -> Vec<String> {
        unimplemented!("S3Storage::list")
    }

    async fn remove(&mut self, key: &str) {
        unimplemented!("S3Storage::remove")
    }

    async fn set(&mut self, key: &str, value: Bytes) {
        self.bucket
            .put_object(key, value.as_ref())
            .await
            .unwrap();
    }
}

#[derive(Debug)]
struct InMemoryStorage {
    data: HashMap<String, Bytes>,
}

impl InMemoryStorage {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

#[async_trait]
impl Storage for InMemoryStorage {
    async fn get(&self, key: &str) -> Option<Bytes> {
        self.data.get(key).cloned()
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
            .map(|k| k.to_string())
            .collect()
    }
    async fn has(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}
