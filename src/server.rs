use std::str;

use async_trait::async_trait;

use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

use tokio_util::sync::CancellationToken;

use tracing::{info, instrument, warn};

use crate::{Context, Service};

pub struct TcpServer {
  pub listener: TcpListener,
}

impl TcpServer {
  async fn listen_for_connections(&mut self, token: CancellationToken) {
    while let Ok((mut stream, _)) = self.listener.accept().await {
      let mut buf = [0; 1024];
      let n = stream
        .read(&mut buf)
        .await
        .expect("read from stream into buffer");
      if let Ok(s) = str::from_utf8(&buf[0..n]) {
        let trimmed = s.trim();
        info!("received data from TCP port: {trimmed}");
        if trimmed == "shutdown" {
          warn!("shutdown received");
          warn!("STOPPING");
          token.cancel();
        }
      }
    }
  }
}

#[async_trait]
impl Service for TcpServer {
  type Context = Context;
  #[instrument(name = "SERVER", skip(self, ctx))]
  async fn invoke(&mut self, ctx: Self::Context) {
    tokio::select! {
      () = self.listen_for_connections(ctx.token.clone()) => (),
      () = ctx.token.cancelled() => (),
    }
    warn!("stopping gracefully");
  }
}
