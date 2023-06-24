use anyhow::{Context, Result};
use futures_util::FutureExt;
use litebin::LiteBin;
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};
use test_log::test;
use tokio::{sync::oneshot, task::JoinHandle};
use tracing::info;

struct TestServer {
    server_task_handle: JoinHandle<()>,
    shutdown_handle: oneshot::Sender<()>,
    addr: SocketAddr,
}

impl TestServer {
    async fn spawn() -> TestServer {
        info!("start server");
        let mut server = LiteBin::serve("[::]:0".parse().unwrap());
        let shutdown_handle = server.take_shutdown_handle().unwrap();
        let addr = server.local_addr();
        let server_task_handle = tokio::spawn(server.map(|res| res.unwrap()));
        info!("server spawned");

        TestServer {
            server_task_handle,
            shutdown_handle,
            addr,
        }
    }

    /// format a URL for the given path
    fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.addr)
    }

    /// Request a graceful shutdown and then wait for shutdown to complete
    async fn shutdown(self) -> Result<()> {
        self.shutdown_handle.send(()).ok();
        self.server_task_handle
            .await
            .context("wait for server shutdown")
    }
}

#[test(tokio::test)]
async fn health_check() -> Result<()> {
    let server = TestServer::spawn().await;
    let status = reqwest::get(server.url("/health")).await?.status();

    assert_eq!(status, 200, "health check failed");

    server.shutdown().await
}

#[test(tokio::test)]
async fn delay_does_delay() -> Result<()> {
    let server = TestServer::spawn().await;
    let before = Instant::now();
    let status = reqwest::get(server.url("/delay")).await?.status();
    assert_eq!(status, 200, "bad status");

    assert!(
        before.elapsed() > Duration::from_secs(1),
        "delay returned too fast: {:?}",
        before.elapsed()
    );

    assert!(
        before.elapsed() < Duration::from_secs_f64(1.1),
        "delay returned too slow: {:?}",
        before.elapsed()
    );

    server.shutdown().await
}

#[test(tokio::test)]
async fn specific_delay_does_delay() -> Result<()> {
    let server = TestServer::spawn().await;
    let before = Instant::now();
    let status = reqwest::get(server.url("/delay/2")).await?.status();
    assert_eq!(status, 200, "bad status");

    assert!(
        before.elapsed() > Duration::from_secs(2),
        "delay returned too fast: {:?}",
        before.elapsed()
    );

    assert!(
        before.elapsed() < Duration::from_secs_f64(2.1),
        "delay returned too slow: {:?}",
        before.elapsed()
    );

    server.shutdown().await
}
