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
    let server = LiteBin::serve(addr, shutdown_signal())
        .await
        .context("build server")?;

    info!("Starting server, listening on {:?}", server.local_addr());

    server.await.context("run server")
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
}
