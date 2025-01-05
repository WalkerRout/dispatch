use std::str;

use actix::prelude::*;
use actix_rt::net::TcpListener;

use tokio::io::AsyncReadExt;
use tokio_util::sync::CancellationToken;

use tracing::{info, instrument, warn, Instrument, Span};

use crate::model::message::ShutdownDispatcher;

#[derive(Default)]
pub struct Server {
  cancel_token: CancellationToken,
}

impl Server {
  pub fn new(cancel_token: CancellationToken) -> Self {
    Self { cancel_token }
  }
}

impl Actor for Server {
  type Context = Context<Self>;

  #[instrument(name = "SERVER", skip(self, ctx))]
  fn started(&mut self, ctx: &mut Self::Context) {
    info!("starting webserver for keypresses...");
    let this = ctx.address();
    async move {
      let listener = TcpListener::bind("127.0.0.1:3599")
        .await
        .expect("open port 3599");
      run_server(listener, this).await;
    }
    .instrument(Span::current())
    .into_actor(self)
    .spawn(ctx);
  }
}

// we might include more messages that the server can handle, so this should stay
// as an Addr<Server>...
async fn run_server(listener: TcpListener, this: Addr<Server>) {
  while let Ok((mut stream, _)) = listener.accept().await {
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
        this.do_send(ShutdownDispatcher);
      }
    }
  }
}

impl Handler<ShutdownDispatcher> for Server {
  type Result = ();

  #[instrument(name = "SERVER", skip(self, _msg, _ctx))]
  fn handle(&mut self, _msg: ShutdownDispatcher, _ctx: &mut Self::Context) -> Self::Result {
    // we are blocking on this cancellation token, letting go will end our actor threads...
    self.cancel_token.cancel();
  }
}
