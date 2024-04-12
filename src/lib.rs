use color_eyre::Result;

use crate::server::HoneypotServer;
use crate::types::Handler;

mod server;
pub mod types;
pub mod favicon;

pub fn run_server(port: u16, handler: Handler) -> Result<()> {
    let server = HoneypotServer::new(port, handler);

    server.start()
}
