//! What messages must be communicated in order to make this program work?

use actix::prelude::*;

use crate::model::key::Key;
use crate::model::script::Script;
use crate::model::keymap::Keymap;

/// Well, we should probably have a way to turn it off...
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct ShutdownDispatcher;

/// Some way to run one of our commands...
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct RunCommand(pub Script);

/// And we need to be able to detect a keypress...
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct KeyDetected(pub Key);

/// We must have some 'globally true' config that maps from keys to scripts...
#[derive(Debug, Message)]
#[rtype(result = "Option<String>")]
pub struct GetScript(pub Key);

/// If we want to support hot reloading, we should have a way to update the current
/// config...
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct UpdateConfig(pub Keymap);
