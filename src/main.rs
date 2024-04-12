use std::sync::Arc;

use color_eyre::eyre::Result;

use mc_honeypot::run_server;
use mc_honeypot::types::{
    Description, Players, Sample, ServerListPingRequest, ServerListPingResponse, Version,
};

fn main() -> Result<()> {
    run_server(25565, Arc::new(handler))?;

    Ok(())
}

fn handler(request: ServerListPingRequest) -> ServerListPingResponse {
    println!("Incoming connection from {}", request.remote_address);
    ServerListPingResponse {
        version: Version {
            name: String::from("1.19.4"),
            protocol: 762,
        },
        players: Players {
            max: 100,
            online: 1,
            sample: vec![Sample {
                name: String::from("Duckulus"),
                id: String::from("b1efb1f9-cb4f-4a07-a6c6-b681e73f9cd1"),
            }],
        },
        description: Description {
            text: String::from("Hallo Welt"),
        },
        favicon: None,
        enforces_secure_chat: true,
        previews_chat: true,
    }
}
