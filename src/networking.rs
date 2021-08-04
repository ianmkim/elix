use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::{Read, Write};

use autodiscover_rs::{self, Method};
use std::thread;
use std::thread::JoinHandle;

type AddrPair = (SocketAddr, SocketAddr);
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync >>;

pub async fn sender(filename:String, addrs:AddrPair) -> Result<()>{
    println!("sender got peer");
    Ok(())
}

pub async fn receiver(code: String, addrs:AddrPair) -> Result<()>{
    println!("receiver got peer");
    Ok(())
}


/// Given a TcpStream object return a SocketAddr pair denoting local and peer address
pub fn tcp_to_addr(stream:TcpStream) -> AddrPair {
    let local = stream.local_addr().unwrap();
    let peer = stream.peer_addr().unwrap();
    (local,peer)
}

pub fn listen_for_peer_response(file:String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let socket = listener.local_addr().unwrap();
    thread::spawn(move || {
        autodiscover_rs::run(&socket, Method::Broadcast("255.255.255.255:1337".parse::<SocketAddr>().unwrap()), |s| {
            let mut rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(sender(file.clone(), tcp_to_addr(s.unwrap()))) {
                Ok(_) => std::process::exit(0),
                Err(e) => { eprintln!("{:?}", e); std::process::exit(0); }
            }
            println!("peer discovered");
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
        autodiscover_rs::run(&socket, Method::Broadcast("255.255.255.255:1337".parse().unwrap()), |s| {

        }).unwrap();
    });

    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next() {
        return Some(stream.unwrap());
    }
    None
}
