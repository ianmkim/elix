use crate::networking::*;
use std::net::{TcpListener,TcpStream, SocketAddr};

use std::thread;
use autodiscover_rs::{self, Method};
use std::io::Read;
use std::io::Write;

use crate::bytes_util::decode_buffer_to_usize;
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
    thread::spawn(move || {
        autodiscover_rs::run(&socket, Method::Broadcast("255.255.255.255:1337".parse::<SocketAddr>().unwrap()), |s| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(sender(file.clone(), tcp_to_addr(s.unwrap()))) {
                Ok(_) => std::process::exit(0),
                Err(e) => { info!("{:?}", e); std::process::exit(0); }
            }
        }).unwrap();
    });
    loop {}
}

/// Function searches for peers on the network using UDP multicasting
/// Returns an Option enums of a TcpStream
pub fn search_for_peer() -> Option<TcpStream> {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let socket = listener.local_addr().unwrap();
    thread::spawn(move || {
        autodiscover_rs::run(&socket, Method::Broadcast("255.255.255.255:1337".parse().unwrap()), |_s| {
        }).unwrap();
    });

    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next() {
        return Some(stream.unwrap());
    }
    None
}

pub fn send_chunk_len(chunk_len:Vec<u8>, addr:SocketAddr){
    let mut stream = TcpStream::connect(addr).expect("Couldn't send the chunk length");
    stream.write(&chunk_len).unwrap();
    loop {
        match stream.read(&mut [0u8;4]) {
            Ok(_) => break,
            Err(e) => info!("Error while reading chunk len: {:?}", e),
        }
    }
}

pub fn receive_chunk_len(addr:SocketAddr) -> usize {
    let listener = TcpListener::bind(&addr).unwrap();
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

    drop(listener);
    chunk_len
}

