use std::net::{Shutdown, TcpStream};

use color_eyre::Result;

use crate::types::{Handler, Request, RequestType, ServerListPingRequest};
use crate::utils::{
    read_byte, read_int, read_unsigned_short, read_utf16_string, write_bytes_to_stream,
};

pub fn handle_legacy_ping(stream: &mut TcpStream, handler: Handler) -> Result<()> {
    let packet_id = read_byte(stream);
    if packet_id.is_err() {
        send_response(stream, handler, 0, String::new(), 0)?;
        return Ok(());
    }

    let payload = read_byte(stream);
    if payload.is_err() {
        send_response(stream, handler, 0, String::new(), 0)?;
        return Ok(());
    }

    let packet_id = read_byte(stream);
    if let Err(_e) = packet_id {
        send_response(stream, handler, 0, String::new(), 0)?;
        return Ok(());
    }

    let channel_len = read_unsigned_short(stream);
    if channel_len.is_err() {
        send_response(stream, handler, 0, String::new(), 0)?;
        return Ok(());
    }

    let _channel = read_utf16_string(stream, channel_len.unwrap()).unwrap_or_default();
    let _len = read_unsigned_short(stream).unwrap_or(0);
    let protocol_version = read_byte(stream).unwrap_or(0) as i32;

    let hostname = read_unsigned_short(stream)
        .map(|len| read_utf16_string(stream, len).unwrap_or_default())
        .unwrap_or_default();

    let port = read_int(stream).unwrap_or(0);

    send_response(stream, handler, protocol_version, hostname, port)?;

    Ok(())
}

fn send_response(
    stream: &mut TcpStream,
    handler: Handler,
    protocol_version: i32,
    hostname: String,
    port: u32,
) -> Result<()> {
    let request = Request {
        remote_address: stream.peer_addr().unwrap(),
        request_type: RequestType::LegacyPing(ServerListPingRequest {
            protocol_version,
            server_address: hostname,
            server_port: port as u16,
        }),
    };
    let response = handler(request);

    let resp_string = format!(
        "§1\0{}\0{}\0{}\0{}\0{}",
        response.version.protocol,
        response.version.name,
        strip_color_codes(&response.description.text),
        response.players.online,
        response.players.max
    );

    let mut resp_buf: Vec<u8> = Vec::new();
    resp_buf.push(0xff);

    let string_len = (resp_string.len() as u16 - 1).to_be_bytes();
    resp_buf.push(string_len[0]);
    resp_buf.push(string_len[1]);

    let utf16_be: Vec<u16> = resp_string
        .encode_utf16()
        .collect::<Vec<u16>>()
        .iter()
        .map(|n| u16::from_be_bytes([(n & 0xFF) as u8, (n >> 8) as u8]))
        .collect();

    unsafe {
        resp_buf.append(&mut utf16_be.align_to::<u8>().1.to_vec());
    }

    write_bytes_to_stream(stream, resp_buf);

    stream.shutdown(Shutdown::Both)?;

    Ok(())
}

fn strip_color_codes(input: &str) -> String {
    let mut sanitized = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '§' {
            i += 2;
        } else {
            sanitized.push(chars[i]);
            i += 1;
        }
    }
    sanitized
}
