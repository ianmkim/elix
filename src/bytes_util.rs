extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use std::io::Cursor;
use std::fs::Metadata;
use std::str;
use log::info;

extern crate rand;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn pad_until_len(mut bytes:Vec<u8>, cap:usize) -> Vec<u8> {
    let to_pad = cap - bytes.len();
    for _ in 0..to_pad {bytes.push(0u8);}
    bytes
}

pub fn generate_code() -> String {
    let rand_string:String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect();
    info!("Random code generated: {}", rand_string);
    rand_string
}

pub fn decode_bytes_to_string(filename_buf:Vec<u8>) -> String {
    let res = str::from_utf8(&filename_buf).unwrap();
    String::from(res.trim_matches(char::from(0)))
}

pub fn encode_string_as_bytes(filename:String) -> Vec<u8> {
    filename.into_bytes()
}

pub fn get_chunk_len(metadata:Metadata, capacity:usize) -> usize {
    let m_len = metadata.len();
    (m_len as f32/ capacity as f32).ceil() as usize
}

pub fn encode_usize_as_vec(usize_var: usize) -> Vec<u8> {
    let mut usize_bytes = [0u8;4];
    usize_bytes.as_mut()
        .write_u32::<LittleEndian>(usize_var as u32)
        .expect("Unable to write usize as bytes");
    usize_bytes.to_vec()
}

pub fn decode_buffer_to_u32(buffer:Vec<u8>) -> u32{
    let mut cursor = Cursor::new(buffer);
    ReadBytesExt::read_u32::<LittleEndian>(&mut cursor).unwrap() as u32
}

pub fn decode_buffer_to_usize(buffer:Vec<u8>) -> usize {
    ReadBytesExt::read_u32::<LittleEndian>(&mut Cursor::new(buffer)).unwrap() as usize
}
