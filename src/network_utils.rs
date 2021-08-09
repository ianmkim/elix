use crate::networking::*;
use std::net::{TcpListener,TcpStream, SocketAddr};

use std::thread;
use autodiscover_rs::{self, Method};
use std::io::Read;
use std::io::Write;

use crate::bytes_util::{
    generate_code,
    decode_buffer_to_usize,
    encode_string_as_bytes,
    decode_bytes_to_string,
};
use log::info;

type AddrPair = (SocketAddr, SocketAddr);

/// Given a TcpStream object return a SocketAddr pair denoting local and peer address
///
pub fn tcp_to_addr(stream:TcpStream) -> AddrPair {
    let local = stream.local_addr().unwrap();
    let peer = stream.peer_addr().unwrap();
    (local,peer)
}

/// Used by the sender to listen for peer responses. When a response from a discovered peer comes,
/// it starts a new tokio task to send the file in chunks
pub fn listen_for_peer_response(file:String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let socket = listener.local_addr().unwrap();
    let rand_string = generate_code();
    println!("To receive {} from another computer, run this command: \n\telix take {}", file,rand_string);
    thread::spawn(move || {
        autodiscover_rs::run(&socket, Method::Broadcast("255.255.255.255:1337".parse::<SocketAddr>().unwrap()), |s| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(sender(file.clone(), tcp_to_addr(s.unwrap()), 1000)) {
                Ok(_) => std::process::exit(0),
                Err(e) => { info!("{:?}", e); std::process::exit(0); }
            }

            let mut s = s.unwrap();
            let mut code_buf= [0u8; 256];
            loop { match s.read_exact(&mut code_buf){
                    Ok(_) => break,
                    Err(e) => info!("Error while reading buffer {:?}", e),}}
            if decode_bytes_to_string(code_buf.to_vec()) == rand_string{
                s.write(&[1u8]).unwrap();
                let rt = tokio::runtime::Runtime::new().unwrap();
                match rt.block_on(sender(file.clone(), tcp_to_addr(s), 1000)) {
                    Ok(_) => std::process::exit(0),
                    Err(e) => { info!("{:?}", e); std::process::exit(0); }
                }
            } else {s.write(&[0u8]).unwrap();}
        }).unwrap();
    });
    loop {}
}

/// Function searches for peers on the network using UDP multicasting
/// Returns an Option enums of a TcpStream
pub fn search_for_peer(code:String) -> Option<TcpStream> {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let socket = listener.local_addr().unwrap();
    thread::spawn(move || {
        autodiscover_rs::run(&socket, Method::Broadcast("255.255.255.255:1337".parse().unwrap()), |_s| {
        }).unwrap();
    });

    let mut incoming = listener.incoming();
    let code_buf = encode_string_as_bytes(code);
    while let Some(stream) = incoming.next() {
        return Some(stream.unwrap());
        let mut stream = stream.unwrap();
        stream.write(&code_buf).unwrap();
        let mut resp_buf = [0u8;1];
        loop { match stream.read(&mut resp_buf) { Ok(_)=>break, Err(e)=>info!("Error while reading {:?}", e), }}
        if resp_buf[0] == 1u8 { return Some(stream); }
    }
    None
}

pub fn send_file_name(filename:String, addr:SocketAddr){
    let mut stream = TcpStream::connect(addr).expect("Couldn't send file name");
    let encoded = encode_string_as_bytes(filename);
    stream.write(&encoded).unwrap();
    loop {
        match stream.read(&mut [0u8;4]){
            Ok(_) => break,
            Err(e) => info!("Error while reading chunk len: {:?}", e),
        }
    }
}

pub fn receive_file_name(listener:&TcpListener) -> String{
    let mut filename = String::new();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut filename_buf = [0u8; 256];
        loop {
            match stream.read(&mut filename_buf){
                Ok(_) => break,
                Err(e) => info!("Error while reading filename {:?}", e),
            }
        }
        filename = decode_bytes_to_string(filename_buf.to_vec());
        info!("Received filename: {}", filename);
        stream.write(&[0u8; 4]).unwrap();
        break
    }
    filename
}

pub fn send_chunk_len(chunk_len:Vec<u8>, addr:SocketAddr){
    let mut stream = TcpStream::connect(addr).expect("Couldn't send the chunk length");
    stream.write(&chunk_len).unwrap();
    loop { match stream.read(&mut [0u8;4]) {
        Ok(_) => break,
        Err(e) => info!("Error while reading chunk len: {:?}", e),
    }}
}

pub fn receive_chunk_len(listener:&TcpListener) -> usize {
    let mut chunk_len = 0;
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut chunk_len_buf = [0u8;4];
        loop {
            match stream.read(&mut chunk_len_buf) {
                Ok(_) => break,
                Err(e) => info!("Error while reading chunk len: {:?}", e),
            }
        }
        chunk_len = decode_buffer_to_usize(chunk_len_buf.to_vec());
        info!("Chunk length {}", chunk_len);
        stream.write(&[0u8; 4]).unwrap();
        break
    }
    chunk_len
}

