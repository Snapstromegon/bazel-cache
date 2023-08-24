use std::sync::Arc;

use anyhow::Context;
use axum::{
    body::StreamBody,
    extract::{BodyStream, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use futures::{stream::StreamExt, Stream};
use tokio::{signal, sync::RwLock};

use crate::storage::Storage;

pub struct Backend {}

impl Backend {
    pub async fn get_action<Store: Storage + ?Sized>(
        Path(path): Path<String>,
        State(cache): State<Arc<RwLock<Store>>>,
    ) -> impl IntoResponse {
        if let Some(data) = cache.read().await.get(&format!("ac/{path}")).await {
            tracing::info!("Responding from cache for ac/{}", path);
            Ok(StreamBody::new(data))
        } else {
            tracing::info!("ac/{} not in cache", path);
            Err(StatusCode::NOT_FOUND)
        }
    }

    pub async fn put_action<Store: Storage + Sync + ?Sized>(
        Path(path): Path<String>,
        State(storage): State<Arc<RwLock<Store>>>,
        body: BodyStream,
    ) -> impl IntoResponse {
        tracing::info!("Storing in cache for ac/{} {:?}", path, body.size_hint());
        if storage.read().await.has(&format!("ac/{path}")).await {
            StatusCode::ACCEPTED
        } else {
            storage
                .write()
                .await
                .set(
                    &format!("ac/{path}"),
                    Box::pin(body.map(|chunk| chunk.context("Mapping error"))),
                )
                .await;
            StatusCode::CREATED
        }
    }

    pub async fn get_item<Store: Storage + ?Sized>(
        Path(path): Path<String>,
        State(storage): State<Arc<RwLock<Store>>>,
    ) -> impl IntoResponse {
        if let Some(data) = storage.read().await.get(&format!("cas/{path}")).await {
            tracing::info!("Responding from cache for cas/{}", path);
            Ok(StreamBody::new(data))
        } else {
            tracing::info!("cas/{} not in cache", path);
            Err(StatusCode::NOT_FOUND)
        }
    }

    pub async fn put_item<Store: Storage + Sync + ?Sized>(
        Path(path): Path<String>,
        State(storage): State<Arc<RwLock<Store>>>,
        body: BodyStream,
    ) -> impl IntoResponse {
        tracing::info!("Storing in cache for cas/{} {:?}", path, body.size_hint());
        if storage.read().await.has(&format!("ac/{path}")).await {
            StatusCode::CREATED
        } else {
            storage
                .write()
                .await
                .set(
                    &format!("cas/{path}"),
                    Box::pin(body.map(|chunk| chunk.context("Mapping error"))),
                )
                .await;
            StatusCode::CREATED
        }
    }

    pub async fn shutdown_signal() {
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
            () = ctrl_c => {},
            () = terminate => {},
        }

        println!("shutting down");
    }
}
