use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
};

use std::fs;

fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    buffer
}

pub fn read_file_once() {
    get_file_as_byte_vec(&String::from("test_data/edited_v2.mp4"));
}

pub fn read_file_chunk() {
    const CAP:usize = 1024 * 1024;
    let filename = String::from("test_data/edited_v2.mp4");
    let file = File::open(&filename).unwrap();
    let mut reader = BufReader::with_capacity(CAP, file);
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut whole_file = vec![0u8; metadata.len() as usize];
    loop {
        let length = {
            let mut buffer = reader.fill_buf().unwrap();
            whole_file.append(&mut buffer.to_vec());
            buffer.len()
        };
        if length == 0 {break;}
        reader.consume(length);
    }
}
