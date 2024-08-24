use async_trait::async_trait;

use tokio::process::Command;
use tokio::sync::mpsc::UnboundedReceiver;

use tokio_util::sync::CancellationToken;

use tracing::{error, info, instrument, warn, Instrument, Span};

use crate::script::Script;
use crate::{Context, Service};

pub struct ScriptRunner {
  pub rx_script: UnboundedReceiver<Script>,
}

impl ScriptRunner {
  async fn listen_for_scripts(&mut self, token: CancellationToken) {
    loop {
      match self.rx_script.recv().await {
        Some(script) => {
          // how would I have the RUNNER instrument follow this spawn?
          tokio::spawn(
            async move {
              let mut cmd = command(&script);
              // spawn and drop handle to child -> will continue running
              match cmd.spawn() {
                Ok(_) => info!("spawned - {script}"),
                Err(e) => error!("failed to spawn - {e} - {script}"),
              }
            }
            .instrument(Span::current()),
          );
        }
        None => {
          error!("failed to receive across rx_script");
          warn!("STOPPING");
          token.cancel();
        }
      }
    }
  }
}

#[async_trait]
impl Service for ScriptRunner {
  type Context = Context;
  #[instrument(name = "RUNNER", skip(self, ctx))]
  async fn invoke(&mut self, ctx: Self::Context) {
    tokio::select! {
      () = self.listen_for_scripts(ctx.token.clone()) => {},
      () = ctx.token.cancelled() => (),
    }
    warn!("stopping gracefully");
  }
}

fn command<C>(cmd: C) -> Command
where
  C: AsRef<str>,
{
  let tokens = command_tokens(cmd);
  if tokens.is_empty() {
    Command::new("")
  } else {
    let mut command = Command::new(&tokens[0]);
    command.args(&tokens[1..]);
    #[cfg(target_family = "windows")]
    {
      use windows::Win32::System::Threading::CREATE_NO_WINDOW;
      command.creation_flags(CREATE_NO_WINDOW.0);
    }
    command
  }
}

fn command_tokens<C>(cmd: C) -> Vec<String>
where
  C: AsRef<str>,
{
  let cmd = cmd.as_ref();

  let mut tokens = Vec::with_capacity(1);
  let mut string_buffer = String::new();

  let mut append_mode = false;
  let mut quote_mode = false;
  let mut quote_mode_ending = false; // to deal with '123''456' -> 123456
  let mut quote_char = ' ';
  let mut escaping = false;

  for c in cmd.chars() {
    if escaping {
      append_mode = true;
      escaping = false;
      string_buffer.push(c);
    } else if c.is_whitespace() {
      if append_mode {
        if quote_mode {
          string_buffer.push(c);
        } else {
          append_mode = false;
          tokens.push(string_buffer);
          string_buffer = String::new();
        }
      } else if quote_mode_ending {
        quote_mode_ending = false;
        tokens.push(string_buffer);
        string_buffer = String::new();
      }
    } else {
      match c {
        '"' | '\'' => {
          if append_mode {
            if quote_mode {
              if quote_char == c {
                append_mode = false;
                quote_mode = false;
                quote_mode_ending = true;
              } else {
                string_buffer.push(c);
              }
            } else {
              quote_mode = true;
              quote_char = c;
            }
          } else {
            append_mode = true;
            quote_mode = true;
            quote_char = c;
          }
        }
        '\\' => {
          escaping = true;
        }
        _ => {
          append_mode = true;
          escaping = false;
          string_buffer.push(c);
        }
      }
    }
  }

  if append_mode || quote_mode_ending {
    tokens.push(string_buffer);
  }

  tokens
}
