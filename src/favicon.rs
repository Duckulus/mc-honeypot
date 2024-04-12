use std::fs;
use std::fs::File;
use std::io::Read;

use base64::prelude::BASE64_STANDARD;
use base64::Engine;

pub fn read_favicon_from_file(file_name: &String) -> std::io::Result<String> {
    let bytes = read_bytes_from_file(file_name)?;
    Ok("data:image/png;base64,".to_owned() + &BASE64_STANDARD.encode(bytes))
}

fn read_bytes_from_file(filename: &String) -> std::io::Result<Vec<u8>> {
    let mut f = File::open(filename)?;
    let metadata = fs::metadata(filename)?;
    let size = metadata.len() as usize;
    let mut buffer = vec![0; size];
    f.read_exact(&mut buffer)?;

    Ok(buffer)
}
