extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use std::io::Cursor;
use std::fs::Metadata;

pub fn get_chunk_len(metadata:Metadata, capacity:usize) -> Vec<u8> {
    let m_len = metadata.len();
    encode_usize_as_vec((m_len as f32/ capacity as f32).ceil() as usize)
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
