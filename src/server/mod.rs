pub mod routes;

use anyhow::{Context as _, Result};
use pin_project::pin_project;
use std::future::Future;
use std::future::IntoFuture as _;
use std::net::SocketAddr;
use std::pin::pin;
use std::pin::Pin;
use tracing::info;

#[pin_project]
pub struct LiteBin {
    #[pin]
    server: Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send>>,
    bound_addr: SocketAddr,
}

impl Future for LiteBin {
    type Output = Result<(), std::io::Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.project().server.poll(cx)
    }
}

impl LiteBin {
    pub fn local_addr(&self) -> SocketAddr {
        self.bound_addr
    }

    pub async fn serve<S>(addr: SocketAddr, shutdown: S) -> Result<LiteBin>
    where
        S: Future<Output = ()> + Send + 'static,
    {
        let app = routes::build();

        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .context("failed to bind listen socket")?;
        let bound_addr = listener
            .local_addr()
            .context("bound socket somehow doesn't have a local address")?;

        let server =
            axum::serve(listener, app.into_make_service()).with_graceful_shutdown(shutdown);

        Ok(LiteBin {
            server: Box::pin(server.into_future()),
            bound_addr,
        })
    }
}
