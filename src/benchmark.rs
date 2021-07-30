use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

use std::fs;

/// Read a file at once
pub fn read_file_once() -> Vec<u8> {
    let filename = &String::from("test_data/edited_v2.mp4");
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    /*
    println!("Uncompressed Length: {}", buffer.len());
    let mut inp = BufReader::new(brotli::CompressorReader::new(buffer.as_slice(), 4096, 1, 22));
    println!("CompressorReader made");
    let mut new_buf:Vec<u8> = Vec::new();
    inp.read_to_end(&mut new_buf);
    println!("Compressed Length:   {}", new_buf.len());
    */

    buffer
}

/// Read a file by breaking it into chunks
pub fn read_file_chunk() -> Vec<u8> {
    // chunk size approximately 1 megabyte
    const CAP:usize = 1024 * 1024;
    // test file 1.2 gigs
    let filename = String::from("test_data/edited_v2.mp4");
    let file = File::open(&filename).unwrap();
    let mut reader = BufReader::with_capacity(CAP, file);
    // the appended final vector of bytes (represented by unsigned 8 bit integer)
    let mut whole_file:Vec<u8> = Vec::new();

    loop {
        let length = {
            let buffer = reader.fill_buf().unwrap();
            // append to the final vector
            whole_file.extend_from_slice(&buffer);
            buffer.len()
        };
        if length == 0 {break;}
        // move onto the next chunk
        reader.consume(length);
    }
    whole_file
}
