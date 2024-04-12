use std::io::{Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4, TcpListener, TcpStream};
use std::str::FromStr;

use color_eyre::eyre::{eyre, Result};
use color_eyre::Report;

fn main() -> Result<()> {
    color_eyre::install()?;

    let listener = TcpListener::bind(SocketAddrV4::new(
        Ipv4Addr::from_str("127.0.0.1").unwrap(),
        25565,
    ))?;

    for stream in listener.incoming() {
        handle_connection(stream?);
    }

    Ok(())
}

fn read_bytes(stream: &mut TcpStream, amount: usize) -> Vec<u8> {
    let mut buf = vec![0; amount];
    stream.read_exact(&mut buf).expect("Error reading bytes");
    buf
}

fn read_byte(stream: &mut TcpStream) -> u8 {
    let mut buf = [0];
    stream.read_exact(&mut buf).expect("Error reading byte");
    buf[0]
}

fn read_unsigned_short(stream: &mut TcpStream) -> u16 {
    (read_byte(stream) as u16) << 8 | read_byte(stream) as u16
}

fn read_long(stream: &mut TcpStream) -> i64 {
    let bytes = read_bytes(stream, 8);
    i64::from_be_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

fn read_varint(stream: &mut TcpStream) -> i32 {
    let mut buf = [0];
    let mut ans = 0;
    for i in 0..4 {
        stream.read_exact(&mut buf).unwrap();
        ans |= ((buf[0] & 0b0111_1111) as i32) << (7 * i);
        if buf[0] & 0b1000_0000 == 0 {
            break;
        }
    }
    ans
}

fn read_string(stream: &mut TcpStream) -> String {
    let len = read_varint(stream) as usize;
    let data: Vec<u8> = read_bytes(stream, len);
    String::from_utf8(data).unwrap()
}

fn write_varint(buffer: &mut Vec<u8>, mut value: i32) {
    if value == 0 {
        buffer.push(0);
        return;
    }
    let mut buf = [0];
    while value != 0 {
        buf[0] = (value & 0b0111_1111) as u8;
        value = (value >> 7) & (i32::MAX >> 6);
        if value != 0 {
            buf[0] |= 0b1000_0000;
        }
        buffer.push(buf[0]);
    }
}

fn write_string(buffer: &mut Vec<u8>, value: String) {
    let mut data = value.into_bytes();
    write_varint(buffer, data.len() as i32);
    buffer.append(&mut data);
}

fn write_bytes_to_stream(stream: &mut TcpStream, bytes: Vec<u8>) {
    stream
        .write_all(bytes.as_slice())
        .expect("Failed to write bytes to stream");
}

fn write_varint_to_stream(stream: &mut TcpStream, mut value: i32) {
    let mut buf = [0];
    if value == 0 {
        stream.write_all(&buf).unwrap();
        return;
    }
    while value != 0 {
        buf[0] = (value & 0b0111_1111) as u8;
        value = (value >> 7) & (i32::MAX >> 6);
        if value != 0 {
            buf[0] |= 0b1000_0000;
        }
        stream.write_all(&buf).unwrap();
    }
}

fn handle_server_list_ping(stream: &mut TcpStream) -> Result<()> {
    // Serverbound Handshake
    let len = read_varint(stream);
    if len == 254 {
        stream.shutdown(Shutdown::Both)?;
        return Err(eyre!("Client sent Legacy Ping. Operation not supported",));
    }

    let _packet_id = read_varint(stream);
    let protocol_version = read_varint(stream);
    let server_address = read_string(stream);
    let server_port = read_unsigned_short(stream);
    let next_state = read_varint(stream);

    if next_state != 1 {
        stream
            .shutdown(Shutdown::Both)
            .expect("Error shutting down stream");
        return Err(eyre!(
            "Client tried joining the Server. Operation not supported",
        ));
    }

    println!("Handshake received. Protocol Version = {protocol_version}, Server Address = {server_address}:{server_port}");

    //Serverbound Status Request
    let _len = read_varint(stream);
    let _packet_id = read_varint(stream);

    println!("Status Request received.");

    // Clientbound Status Response
    let mut resp_buf: Vec<u8> = Vec::new();
    write_varint(&mut resp_buf, 0);
    let payload = String::from(
        r#"{"version":{"name":"1.20.4","protocol":765},"players":{"max":10000000000000000000,"online":5,"sample":[{"name":"thinkofdeath","id":"4566e69f-c907-48ee-8d71-d7ba5aa00d20"}]},"description":{"text":"Hello world"},"enforcesSecureChat":true,"previewsChat":true}"#,
    );
    write_string(&mut resp_buf, payload);

    write_varint_to_stream(stream, resp_buf.len() as i32);
    write_bytes_to_stream(stream, resp_buf);
    println!("Status Response Sent");

    // Serverbound Ping Request
    let mut len = [0];
    match stream.read(&mut len) {
        Ok(n) => {
            if n == 0 {
                return Ok(());
            }
        }
        Err(e) => {
            return Err(Report::from(e));
        }
    };
    let _packet_id = read_varint(stream);
    let payload = read_long(stream);

    println!("Ping Request received. Payload = {payload}");

    //Clientbound Ping Response
    let mut resp_buf: Vec<u8> = Vec::new();
    write_varint(&mut resp_buf, 1);
    resp_buf.append(&mut payload.to_be_bytes().to_vec());
    write_varint_to_stream(stream, resp_buf.len() as i32);
    write_bytes_to_stream(stream, resp_buf);
    println!("Pong Response sent.");

    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    println!("Incoming connection from {}", stream.peer_addr().unwrap());
    std::thread::spawn(move || {
        if let Err(report) = handle_server_list_ping(&mut stream) {
            eprintln!("{}", report)
        }
    });
}
