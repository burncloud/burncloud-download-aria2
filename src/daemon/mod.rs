pub(crate) mod platform;
pub(crate) mod binary;
pub(crate) mod process;
pub(crate) mod monitor;
mod orchestrator;

#[cfg(test)]
mod tests;

pub use orchestrator::{Aria2Daemon, DaemonConfig};
