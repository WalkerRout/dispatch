#![allow(async_fn_in_trait)]

use std::sync::Arc;

use async_trait::async_trait;

use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

pub mod filter;
pub mod key;
pub mod keymap;
pub mod listener;
pub mod monitor;
pub mod runner;
pub mod script;

#[async_trait]
pub trait Service {
  type Context: Send + Sync + 'static;
  /// A service can be invoked with some context
  async fn invoke(&mut self, ctx: Self::Context);
}

pub trait Application {
  type Context: Clone + Send + Sync + 'static;
  /// An application can be invoked with some set of services
  async fn invoke_all(
    &self,
    ctx: Self::Context,
    services: impl IntoIterator<Item = Box<dyn Service<Context = Self::Context> + Send>>,
  ) {
    let tracker = TaskTracker::new();
    for mut service in services {
      let ctx = ctx.clone();
      tracker.spawn(async move {
        service.invoke(ctx).await;
      });
    }
    tracker.close();
    tracker.wait().await;
  }
}

#[derive(Debug, Clone)]
pub struct Context {
  pub token: CancellationToken,
  pub table: Arc<RwLock<keymap::Keymap>>,
}
