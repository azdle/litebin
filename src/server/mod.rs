pub mod routes;

use anyhow::Result;
use pin_project::pin_project;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::pin;
use std::pin::Pin;
use tokio::sync::oneshot;
use tracing::info;

#[pin_project]
pub struct LiteBin {
    #[pin]
    server: Pin<Box<dyn Future<Output = Result<(), hyper::Error>> + Send>>,
    bound_addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
}

impl Future for LiteBin {
    type Output = Result<(), hyper::Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.project().server.poll(cx)
    }
}

impl LiteBin {
    pub fn shutdown(self) -> Result<()> {
        if let Some(shutdown) = self.shutdown {
            shutdown
                .send(())
                .map_err(|()| anyhow::anyhow!("failed to send"))
        } else {
            Err(anyhow::anyhow!("shutdown handle gone"))
        }
    }

    pub fn take_shutdown_handle(&mut self) -> Option<oneshot::Sender<()>> {
        self.shutdown.take()
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.bound_addr
    }

    pub fn serve(addr: SocketAddr) -> LiteBin {
        let app = routes::build();

        let (shutdown_sender, shutdown_receiver) = oneshot::channel();

        info!("Starting server, listening on {addr:?}");

        let server = axum::Server::bind(&addr).serve(app.into_make_service());

        let bound_addr = server.local_addr();

        let server = server.with_graceful_shutdown(async {
            shutdown_receiver.await.ok();
        });

        LiteBin {
            server: Box::pin(server),
            bound_addr,
            shutdown: Some(shutdown_sender),
        }
    }
}
