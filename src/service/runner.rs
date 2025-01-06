use std::future::Future;
use std::pin::Pin;

use actix::prelude::*;

use tokio::process::Command;

use tracing::{error, info, instrument, Instrument, Span};

use crate::model::message::RunCommand;
use crate::model::script::Script;

/// A runner receives some shell scripts and invokes them in a child process on
/// the host OS
#[derive(Default)]
pub struct Runner;

impl Runner {
  pub fn new() -> Self {
    Self
  }

  fn dispatch_scripts(&mut self, script: Script) -> Pin<Box<dyn Future<Output = ()>>> {
    Box::pin(async move {
      let mut cmd = command(&script);
      // tokio does not have great guarantees on reaping zombie processes, so we explicitly
      // await spawned children to guarantee resource cleanup
      #[cfg(target_family = "unix")]
      {
        info!("spawning child");
        match cmd.spawn() {
          Ok(mut child) => match child.wait().await {
            Ok(_) => (),
            Err(e) => error!("failed to run - {e}"),
          },
          Err(e) => error!("failed to spawn - {e} - {script}"),
        }
      }
      // spawn and drop handle to child -> will continue running
      #[cfg(target_family = "windows")]
      {
        match cmd.spawn() {
          Ok(_) => info!("spawned - {script}"),
          Err(e) => error!("failed to spawn - {e} - {script}"),
        }
      }
    })
  }
}

impl Actor for Runner {
  type Context = Context<Self>;
}

impl Handler<RunCommand> for Runner {
  type Result = ();

  #[instrument(name = "RUNNER", skip(self, msg, ctx))]
  fn handle(&mut self, msg: RunCommand, ctx: &mut Context<Self>) -> Self::Result {
    self
      .dispatch_scripts(msg.0)
      .instrument(Span::current())
      .into_actor(self)
      .spawn(ctx);
  }
}

fn command<C>(cmd: C) -> Command
where
  C: AsRef<str>,
{
  let cmd_str = cmd.as_ref();
  let tokens = command_tokens(cmd_str);
  if tokens.is_empty() {
    Command::new("")
  } else {
    #[cfg(target_family = "unix")]
    {
      let mut command = Command::new("sh");
      command.arg("-c");
      command.arg(cmd_str);
      command
    }
    #[cfg(target_family = "windows")]
    {
      use windows::Win32::System::Threading::CREATE_NO_WINDOW;
      let mut command = Command::new(&tokens[0]);
      command.args(&tokens[1..]);
      command.creation_flags(CREATE_NO_WINDOW.0);
      command
    }
  }
}

fn command_tokens(cmd: &str) -> Vec<String> {
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
