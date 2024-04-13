use std::io::{Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4, TcpListener, TcpStream};
use std::str::FromStr;

use color_eyre::eyre::{eyre, Result};
use color_eyre::Report;

use crate::types::{Handler, ServerListPingRequest};

pub struct HoneypotServer {
    port: u16,
    handler: Handler,
}

impl HoneypotServer {
    pub fn new(port: u16, handler: Handler) -> Self {
        Self { port, handler }
    }

    pub fn start(self) -> Result<()> {
        color_eyre::install()?;

        let listener = TcpListener::bind(SocketAddrV4::new(
            Ipv4Addr::from_str("0.0.0.0").unwrap(),
            self.port,
        ))?;

        println!("Started Server on port {}", self.port);

        for stream in listener.incoming() {
            Self::handle_connection(stream?, &self.handler);
        }

        Ok(())
    }

    fn handle_connection(mut stream: TcpStream, handler: &Handler) {
        let cloned = handler.clone();
        std::thread::spawn(move || {
            if let Err(report) = Self::handle_server_list_ping(&mut stream, cloned) {
                eprintln!("{}", report)
            }
        });
    }

    fn handle_server_list_ping(stream: &mut TcpStream, handler: Handler) -> Result<()> {
        // Serverbound Handshake
        let len = Self::read_varint(stream);
        if len == 254 {
            stream.shutdown(Shutdown::Both)?;
            return Err(eyre!("Client sent Legacy Ping. Operation not supported",));
        }

        let _packet_id = Self::read_varint(stream);
        let protocol_version = Self::read_varint(stream);
        let server_address = Self::read_string(stream);
        let server_port = Self::read_unsigned_short(stream);
        let next_state = Self::read_varint(stream);

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
        let _len = Self::read_varint(stream);
        let _packet_id = Self::read_varint(stream);

        println!("Status Request received.");

        let request = ServerListPingRequest {
            remote_address: stream.peer_addr().unwrap(),
            protocol_version,
            server_address,
            server_port,
        };

        let response = handler(request);
        let response_json = serde_json::to_string(&response)?;

        // Clientbound Status Response
        let mut resp_buf: Vec<u8> = Vec::new();
        Self::write_varint(&mut resp_buf, 0);
        Self::write_string(&mut resp_buf, response_json);

        Self::write_varint_to_stream(stream, resp_buf.len() as i32);
        Self::write_bytes_to_stream(stream, resp_buf);
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
        let _packet_id = Self::read_varint(stream);
        let payload = Self::read_long(stream);

        println!("Ping Request received. Payload = {payload}");

        //Clientbound Ping Response
        let mut resp_buf: Vec<u8> = Vec::new();
        Self::write_varint(&mut resp_buf, 1);
        resp_buf.append(&mut payload.to_be_bytes().to_vec());
        Self::write_varint_to_stream(stream, resp_buf.len() as i32);
        Self::write_bytes_to_stream(stream, resp_buf);
        println!("Pong Response sent.");

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
        (Self::read_byte(stream) as u16) << 8 | Self::read_byte(stream) as u16
    }

    fn read_long(stream: &mut TcpStream) -> i64 {
        let bytes = Self::read_bytes(stream, 8);
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
        let len = Self::read_varint(stream) as usize;
        let data: Vec<u8> = Self::read_bytes(stream, len);
        String::from_utf8(data).unwrap_or_default()
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
        Self::write_varint(buffer, data.len() as i32);
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
}
