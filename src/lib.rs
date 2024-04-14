use color_eyre::Result;

use crate::server::HoneypotServer;
use crate::types::Handler;

pub mod color;
pub mod favicon;
mod server;
pub mod types;
pub mod utils;
pub mod webhook;

pub fn run_server(port: u16, handler: Handler) -> Result<()> {
    let server = HoneypotServer::new(port, handler);

    server.start()
}
