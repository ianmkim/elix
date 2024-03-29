extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use std::io::Cursor;
use std::fs::Metadata;
use std::str;
use log::info;

extern crate rand;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

/// Pads input byte vector until the given capacity size and returns it
pub fn pad_until_len(mut bytes:Vec<u8>, cap:usize) -> Vec<u8> {
    let to_pad = cap - bytes.len();
    for _ in 0..to_pad {bytes.push(0u8);}
    bytes
}

/// Randomly generates a 5 character code
/// should have 62^5 combinations, which is plenty enough
/// since this utility will only be used through local networks
pub fn generate_code() -> String {
    let rand_string:String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect();
    info!("Random code generated: {}", rand_string);
    rand_string
}

/// Decodes a byte vector to string and trims the trailing 0 bytes
pub fn decode_bytes_to_string(filename_buf:Vec<u8>) -> String {
    let res = str::from_utf8(&filename_buf).unwrap();
    String::from(res.trim_matches(char::from(0)))
}

/// Encodes a string to a byte vector using built in .into_bytes() function
pub fn encode_string_as_bytes(filename:String) -> Vec<u8> {
    filename.into_bytes()
}

/// Given a capacity and total size of file to send, returns the relevant chunk size
pub fn get_chunk_len(metadata:Metadata, capacity:usize) -> usize {
    let m_len = metadata.len();
    (m_len as f32/ capacity as f32).ceil() as usize
}

/// Encodes a usize as a byte vector using little endian scheme
pub fn encode_usize_as_vec(usize_var: usize) -> Vec<u8> {
    let mut usize_bytes = [0u8;4];
    usize_bytes.as_mut()
        .write_u32::<LittleEndian>(usize_var as u32)
        .expect("Unable to write usize as bytes");
    usize_bytes.to_vec()
}

/// Deodes a byte vector as a unsigned integer 32bits using a little endan scheme
pub fn decode_buffer_to_u32(buffer:Vec<u8>) -> u32{
    let mut cursor = Cursor::new(buffer);
    ReadBytesExt::read_u32::<LittleEndian>(&mut cursor).unwrap() as u32
}

/// Given a byte vector buffer, decode it and return the usize content 
pub fn decode_buffer_to_usize(buffer:Vec<u8>) -> usize {
    ReadBytesExt::read_u32::<LittleEndian>(&mut Cursor::new(buffer)).unwrap() as usize
}
