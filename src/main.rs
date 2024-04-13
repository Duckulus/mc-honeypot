use std::sync::Arc;

use clap::{command, Parser};
use color_eyre::eyre::Result;
use log::LevelFilter;
use simple_logger::{set_up_color_terminal, SimpleLogger};

use mc_honeypot::favicon::read_favicon_from_file;
use mc_honeypot::run_server;
use mc_honeypot::types::{
    Description, Handler, Players, Request, RequestType, ServerListPingResponse, Version,
};
use mc_honeypot::webhook::log_ping_to_webhook;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        help = "The port the honeypot will listen on",
        default_value = "25565"
    )]
    port: u16,
    #[arg(
        short,
        long,
        help = "The version string displayed by the Client",
        default_value = "1.20.4"
    )]
    version_string: String,
    #[arg(
        long,
        help = "This is used by clients to determine if it is compatible with our server",
        default_value = "765"
    )]
    protocol_version: i32,
    #[arg(
        short,
        long,
        help = "The displayed maximum player count",
        default_value = "100"
    )]
    max_players: i32,
    #[arg(
        short,
        long,
        help = "The displayed online player count",
        default_value = "0"
    )]
    online_players: i32,
    #[arg(
        long,
        help = "The displayed \"Message of the Day\"",
        default_value = "§aHello, World"
    )]
    motd: String,
    #[arg(
        short,
        long,
        help = "Path to png image which is displayed as the server icon. Needs to be 64x64 pixels in size"
    )]
    icon_file: Option<String>,
    #[arg(short, long, help = "URL of discord webhook to send logs to")]
    webhook_url: Option<String>,
}

fn main() -> Result<()> {
    set_up_color_terminal();
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let args = Args::parse();

    run_server(args.port, get_handler(args.clone()))?;

    Ok(())
}

fn get_handler(args: Args) -> Handler {
    let favicon = args.icon_file.map(|s| match read_favicon_from_file(&s) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    });

    let cloned_webhook_url = args.webhook_url.clone();

    Arc::new(move |request: Request| {
        if let Some(url) = &cloned_webhook_url {
            log_ping_to_webhook(url, &request.remote_address, &request.request_type);
        }
        match request.request_type {
            RequestType::JOIN => {
                log::info!(
                    "[{}] Player tried joining the server",
                    request.remote_address
                );
            }
            RequestType::LegacyPing(req) => {
                log::info!(
                    "[{}] Received Legacy Ping Request [{:?}]",
                    request.remote_address,
                    req
                )
            }
            RequestType::ModernPing(req) => {
                log::info!(
                    "[{}] Received Ping Request [{:?}]",
                    request.remote_address,
                    req
                )
            }
        };
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
            favicon: favicon.clone(),
            enforces_secure_chat: true,
            previews_chat: true,
        }
    })
}
