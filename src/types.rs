use std::net::SocketAddr;
use std::sync::Arc;

use serde::Serialize;

pub type Handler = Arc<dyn Fn(Request) -> ServerListPingResponse + Send + Sync + 'static>;

pub struct Request {
    pub request_type: RequestType,
    pub remote_address: SocketAddr,
}

pub enum RequestType {
    Join(Sample),
    ModernPing(ServerListPingRequest),
    LegacyPing(ServerListPingRequest),
}

#[derive(Debug)]
pub struct ServerListPingRequest {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
}

#[derive(Serialize)]
pub struct ServerListPingResponse {
    pub version: Version,
    pub players: Players,
    pub description: Description,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    #[serde(rename(serialize = "enforcesSecureChat"))]
    pub enforces_secure_chat: bool,
    #[serde(rename(serialize = "previewsChat"))]
    pub previews_chat: bool,
}

#[derive(Serialize)]
pub struct Version {
    pub name: String,
    pub protocol: i32,
}

#[derive(Serialize)]
pub struct Players {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<Sample>,
}

#[derive(Serialize, Clone)]
pub struct Sample {
    pub name: String,
    pub id: String,
}

#[derive(Serialize)]
pub struct Description {
    pub text: String,
}
