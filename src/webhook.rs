use std::net::SocketAddr;

use serde::Serialize;

use crate::color::RgbColor;
use crate::types::RequestType;

#[derive(Serialize)]
struct WebhookPayload {
    embeds: Vec<Embed>,
}

impl WebhookPayload {
    fn new(message: String, color: i32) -> WebhookPayload {
        WebhookPayload {
            embeds: vec![Embed {
                title: String::from("Ping!"),
                description: message,
                color,
            }],
        }
    }
}

#[derive(Serialize)]
struct Embed {
    title: String,
    description: String,
    color: i32,
}

pub fn log_ping_to_webhook(url: &str, address: &SocketAddr, request_type: &RequestType) {
    let information = match request_type {
        RequestType::JOIN => String::from("Player tried joining the Server"),
        RequestType::LegacyPing(ref req) => format!("Player sent legacy Ping: {:?}", req),
        RequestType::ModernPing(ref req) => format!("Player sent reqular Ping: {:?}", req),
    };
    let msg = format!(
        "Received Ping from [`{}`](https://{}/)\n\n {}",
        address, address, information
    );
    send_webhook_message(
        url.to_owned(),
        msg,
        get_color_from_request_type(request_type),
    )
}

fn get_color_from_request_type(request_type: &RequestType) -> i32 {
    match request_type {
        RequestType::JOIN => RgbColor::new(250, 20, 20).rgb(),
        RequestType::LegacyPing(_) => RgbColor::new(220, 150, 20).rgb(),
        RequestType::ModernPing(_) => RgbColor::new(20, 250, 20).rgb(),
    }
}

fn send_webhook_message(url: String, message: String, color: i32) {
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let res = client
            .post(url)
            .json(&WebhookPayload::new(message, color))
            .send();

        if let Err(e) = res {
            log::error!("There was an error executing discord webhook {}", e);
        }
    });
}
