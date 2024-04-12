use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:25565")?;

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

fn read_varint(stream: &mut TcpStream) -> i32 {
    let mut buf = [0];
    let mut ans = 0;
    for i in 0..4 {
        stream.read_exact(&mut buf).unwrap();
        ans |= ((buf[0] & 0b0111_1111) as i32) << 7 * i;
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
    let mut buf = [0];
    while value != 0 {
        buf[0] = (value & 0b0111_1111) as u8;
        value = (value >> 7) & (i32::max_value() >> 6);
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
    while value != 0 {
        buf[0] = (value & 0b0111_1111) as u8;
        value = (value >> 7) & (i32::max_value() >> 6);
        if value != 0 {
            buf[0] |= 0b1000_0000;
        }
        stream.write_all(&buf).unwrap();
    }
}

fn handle_connection(mut stream: TcpStream) {
    println!("Incoming connection from {}", stream.peer_addr().unwrap());
    std::thread::spawn(move || {
        // Serverbound Handshake
        let len = read_varint(&mut stream);
        if len == 254 {
            println!("Received Legacy Ping. Aborting");
            stream
                .shutdown(Shutdown::Both)
                .expect("Error shutting down stream");
            return;
        }

        let packet_id = read_varint(&mut stream);
        let protocol_version = read_varint(&mut stream);
        let server_address = read_string(&mut stream);
        let server_port = read_unsigned_short(&mut stream);
        let next_state = read_varint(&mut stream);

        if next_state != 1 {
            println!("Player tried joining the server. Aborting");
            stream
                .shutdown(Shutdown::Both)
                .expect("Error shutting down stream");
            return;
        }

        dbg!(len);
        dbg!(packet_id);
        dbg!(protocol_version);
        dbg!(server_address);
        dbg!(server_port);
        dbg!(next_state);

        //Serverbound Status Request
        let len = read_varint(&mut stream);
        let packet_id = read_varint(&mut stream);
        dbg!(len);
        dbg!(packet_id);

        // Clientbound Status Response
        let mut resp_buf: Vec<u8> = Vec::new();
        resp_buf.push(0);
        let payload = String::from(
            r#"{"version":{"name":"1.20.4","protocol":762},"players":{"max":100,"online":5,"sample":[{"name":"thinkofdeath","id":"4566e69f-c907-48ee-8d71-d7ba5aa00d20"}]},"description":{"text":"Hello world"},"enforcesSecureChat":true,"previewsChat":true}"#,
        );
        write_string(&mut resp_buf, payload);

        write_varint_to_stream(&mut stream, resp_buf.len() as i32);
        write_bytes_to_stream(&mut stream, resp_buf);
    });
}
