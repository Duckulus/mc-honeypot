use std::sync::Arc;

use clap::{command, Parser};
use color_eyre::eyre::Result;

use mc_honeypot::run_server;
use mc_honeypot::types::{
    Description, Handler, Players, ServerListPingRequest, ServerListPingResponse, Version,
};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "25565")]
    port: u16,
    #[arg(short, long, default_value = "1.20.4")]
    version_string: String,
    #[arg(long, default_value = "765")]
    protocol_version: i32,
    #[arg(short, long, default_value = "100")]
    max_players: i32,
    #[arg(short, long, default_value = "0")]
    online_players: i32,
    #[arg(long, default_value = "§aHello, World")]
    motd: String,
    #[arg(short, long)]
    icon_file: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    run_server(args.port, get_handler(args.clone()))?;

    Ok(())
}

fn get_handler(args: Args) -> Handler {
    Arc::new(move |request: ServerListPingRequest| {
        println!("Incoming connection from {}", request.remote_address);
        ServerListPingResponse {
            version: Version {
                name: args.version_string.clone(),
                protocol: args.protocol_version,
            },
            players: Players {
                max: args.max_players,
                online: args.online_players,
                sample: vec![],
            },
            description: Description {
                text: args.motd.clone(),
            },
            favicon: None,
            enforces_secure_chat: true,
            previews_chat: true,
        }
    })
}
