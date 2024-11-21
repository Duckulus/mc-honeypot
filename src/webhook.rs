use std::net::SocketAddr;
use std::sync::mpsc::{channel, Sender};

use serde::Serialize;
use timer::{Guard, Timer};

use crate::color::RgbColor;
use crate::types::RequestType;

const MAX_EMBEDS_PER_MESSAGE: usize = 10;

#[derive(Serialize)]
struct WebhookPayload {
    embeds: Vec<Embed>,
}

impl WebhookPayload {
    fn new(embeds: &[Embed]) -> WebhookPayload {
        WebhookPayload {
            embeds: embeds.to_vec(),
        }
    }
}

#[derive(Serialize, Clone)]
struct Embed {
    title: String,
    description: String,
    color: i32,
}

struct WebhookBuffer {
    pub embeds: Vec<Embed>,
    pub url: String,
}

impl WebhookBuffer {
    fn new(url: String) -> WebhookBuffer {
        WebhookBuffer {
            embeds: vec![],
            url,
        }
    }

    fn add_message(&mut self, embed: Embed) {
        self.embeds.push(embed);
        if self.embeds.len() >= MAX_EMBEDS_PER_MESSAGE {
            self.flush();
        }
    }

    fn flush(&mut self) {
        if !self.embeds.is_empty() {
            let payload = WebhookPayload::new(&self.embeds);
            send_webhook_payload(self.url.clone(), payload);
            self.embeds.clear();
        }
    }
}

enum Message {
    Flush,
    AddEmbed(Embed),
}

#[allow(unused)]
pub struct BufferedWebhookClient {
    timer: Timer,
    guard: Guard,
    transmitter: Sender<Message>,
}

impl BufferedWebhookClient {
    pub fn new(url: String) -> BufferedWebhookClient {
        let timer = Timer::new();
        let (tx, rx) = channel();

        let tx1 = tx.clone();
        let guard = timer.schedule_repeating(chrono::Duration::seconds(5), move || {
            if let Err(e) = tx1.send(Message::Flush) {
                log::error!("Error sending Flush message to Receiver Thread {}", e);
            }
        });

        std::thread::spawn(move || {
            let mut buf = WebhookBuffer::new(url);
            for received in rx {
                match received {
                    Message::Flush => buf.flush(),
                    Message::AddEmbed(embed) => buf.add_message(embed),
                }
            }
        });

        BufferedWebhookClient {
            timer,
            guard,
            transmitter: tx,
        }
    }

    pub fn send_flush(&mut self) {
        if let Err(e) = self.transmitter.send(Message::Flush) {
            log::error!("Error sending Flush message to Receiver Thread {}", e);
        }
    }

    pub fn send(&self, address: &SocketAddr, request_type: &RequestType) {
        if let Err(e) = self
            .transmitter
            .send(Message::AddEmbed(build_embed(address, request_type)))
        {
            log::error!("Error sending message to Receiver Thread {}", e);
        }
    }
}

impl Drop for BufferedWebhookClient {
    fn drop(&mut self) {
        self.send_flush();
    }
}

fn build_embed(address: &SocketAddr, request_type: &RequestType) -> Embed {
    let information = match request_type {
        RequestType::Join(ref req) => format!(
            "[`{}` (`{}`)](https://namemc.com/profile/{}) tried joining the Server",
            req.name, req.id, req.id
        ),
        RequestType::LegacyPing(ref req) => format!("Player sent legacy Ping: {:?}", req),
        RequestType::ModernPing(ref req) => format!("Player sent regular Ping: {:?}", req),
    };
    let msg = format!(
        "Received Ping from [`{}`](https://{}/)\n\n {}",
        address, address, information
    );
    Embed {
        title: String::from("Ping!"),
        description: msg,
        color: get_color_from_request_type(request_type),
    }
}

fn get_color_from_request_type(request_type: &RequestType) -> i32 {
    match request_type {
        RequestType::Join(_) => RgbColor::new(250, 20, 20).rgb(),
        RequestType::LegacyPing(_) => RgbColor::new(220, 150, 20).rgb(),
        RequestType::ModernPing(_) => RgbColor::new(20, 250, 20).rgb(),
    }
}

fn send_webhook_payload(url: String, payload: WebhookPayload) {
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let res = client.post(url).json(&payload).send();

        if let Err(e) = res {
            log::error!("There was an error executing discord webhook {}", e);
        }
    });
}
