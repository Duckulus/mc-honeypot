use std::io::Read;
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4, TcpListener, TcpStream};
use std::str::FromStr;
use std::time::Duration;

use color_eyre::eyre::Result;
use color_eyre::Report;

use crate::server::legacy::handle_legacy_ping;
use crate::types::{Handler, Request, RequestType, ServerListPingRequest};
use crate::utils::{
    read_long, read_unsigned_short, read_utf8_string, read_varint, write_bytes_to_stream,
    write_utf8_string, write_varint, write_varint_to_stream,
};

pub mod legacy;

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
        stream.set_read_timeout(Some(Duration::from_millis(200)))?;

        let mut buf: [u8; 1] = [0];
        stream.peek(&mut buf)?;
        if buf[0] == 0xFE {
            handle_legacy_ping(stream, handler)?;
            return Ok(());
        }

        // Serverbound Handshake
        let _len = read_varint(stream)?;
        let _packet_id = read_varint(stream)?;
        let protocol_version = read_varint(stream)?;
        let server_address = read_utf8_string(stream)?;
        let server_port = read_unsigned_short(stream).expect("Expected Server Port");
        let next_state = read_varint(stream)?;

        if next_state == 2 {
            stream
                .shutdown(Shutdown::Both)
                .expect("Error shutting down stream");

            handler(Request {
                request_type: RequestType::JOIN,
                remote_address: stream.peer_addr().unwrap(),
            });
            return Ok(());
        }

        //Serverbound Status Request
        let _len = read_varint(stream);
        let _packet_id = read_varint(stream);

        let request = Request {
            remote_address: stream.peer_addr().unwrap(),
            request_type: RequestType::ModernPing(ServerListPingRequest {
                protocol_version,
                server_address,
                server_port,
            }),
        };

        let response = handler(request);
        let response_json = serde_json::to_string(&response)?;

        // Clientbound Status Response
        let mut resp_buf: Vec<u8> = Vec::new();
        write_varint(&mut resp_buf, 0);
        write_utf8_string(&mut resp_buf, response_json);

        write_varint_to_stream(stream, resp_buf.len() as i32);
        write_bytes_to_stream(stream, resp_buf);

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
        let _packet_id = read_varint(stream)?;
        let payload = read_long(stream)?;

        //Clientbound Ping Response
        let mut resp_buf: Vec<u8> = Vec::new();
        write_varint(&mut resp_buf, 1);
        resp_buf.append(&mut payload.to_be_bytes().to_vec());
        write_varint_to_stream(stream, resp_buf.len() as i32);
        write_bytes_to_stream(stream, resp_buf);

        stream.shutdown(Shutdown::Both)?;

        Ok(())
    }
}
