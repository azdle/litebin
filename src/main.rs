use anyhow::{Context, Result};
use litebin::LiteBin;
use tokio::signal;
use tracing::info;

fn init_tracing() {
    use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

    let log_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    fmt().pretty().with_env_filter(log_filter).init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let addr = "[::]:3742".parse().unwrap();
    let mut server = LiteBin::serve(addr);

    // Turn a SIGINT into a graceful shutdown
    let server_shutdown = server.take_shutdown_handle().unwrap();
    tokio::spawn(async {
        signal::ctrl_c().await.unwrap();
        info!("[Ctrl-C] Shutting Down");
        server_shutdown.send(()).unwrap();
    });

    server.await.context("run server")
}
