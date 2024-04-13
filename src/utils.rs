use std::io::{Read, Write};
use std::net::TcpStream;

use color_eyre::Result;

pub fn read_bytes(stream: &mut TcpStream, amount: usize) -> Vec<u8> {
    let mut buf = vec![0; amount];
    stream.read_exact(&mut buf).expect("Error reading bytes");
    buf
}

pub fn read_byte(stream: &mut TcpStream) -> Result<u8> {
    let mut buf = [0];
    stream.read_exact(&mut buf)?;
    Ok(buf[0])
}

pub fn read_unsigned_short(stream: &mut TcpStream) -> Result<u16> {
    Ok((read_byte(stream)? as u16) << 8 | read_byte(stream)? as u16)
}

pub fn read_short_le(stream: &mut TcpStream) -> Result<u16> {
    Ok(u16::from_le_bytes([read_byte(stream)?, read_byte(stream)?]))
}

pub fn read_int(stream: &mut TcpStream) -> Result<u32> {
    Ok(u32::from_be_bytes([
        read_byte(stream)?,
        read_byte(stream)?,
        read_byte(stream)?,
        read_byte(stream)?,
    ]))
}

pub fn read_long(stream: &mut TcpStream) -> i64 {
    let bytes = read_bytes(stream, 8);
    i64::from_be_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

pub fn read_varint(stream: &mut TcpStream) -> i32 {
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

pub fn read_utf8_string(stream: &mut TcpStream) -> String {
    let len = read_varint(stream) as usize;
    let data: Vec<u8> = read_bytes(stream, len);
    String::from_utf8(data).unwrap_or_default()
}

pub fn read_utf16_string(stream: &mut TcpStream, chars: u16) -> Result<String> {
    let mut shorts = Vec::new();
    for _ in 0..chars {
        shorts.push(read_unsigned_short(stream)?);
    }
    Ok(String::from_utf16(shorts.as_slice()).expect("Expected UTF-16 String"))
}

pub fn write_varint(buffer: &mut Vec<u8>, mut value: i32) {
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

pub fn write_utf8_string(buffer: &mut Vec<u8>, value: String) {
    let mut data = value.into_bytes();
    write_varint(buffer, data.len() as i32);
    buffer.append(&mut data);
}

pub fn write_bytes_to_stream(stream: &mut TcpStream, bytes: Vec<u8>) {
    stream
        .write_all(bytes.as_slice())
        .expect("Failed to write bytes to stream");
}

pub fn write_varint_to_stream(stream: &mut TcpStream, value: i32) {
    let mut buf = Vec::new();
    write_varint(&mut buf, value);
    write_bytes_to_stream(stream, buf);
}
