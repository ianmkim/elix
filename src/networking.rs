use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io;

// TODO benchmark autodiscover_rs and rewrite if necessary
use autodiscover_rs::{self, Method};

/// Used only for debugging, prints the address of the peer
fn handle_client(stream: io::Result<TcpStream>) {
    println!("Received Connection From {:?}", stream.unwrap().peer_addr());
}

/// Function searches for peers on the network using UDP multicasting
/// Returns an Option enums of a TcpStream
pub fn search_for_peer() -> Option<TcpStream> {
    let listener = TcpListener::bind(":::0").unwrap();
    let socket = listener.local_addr().unwrap();
    thread::spawn(move || {
        autodiscover_rs::run(&socket, Method::Multicast("[ff0e::1]:1337".parse().unwrap()), |s| {
            thread::spawn(|| handle_client(s));
        }).unwrap();
    });

    let mut incoming = listener.incoming(); 
    while let Some(stream) = incoming.next() {
        return Some(stream.unwrap());
    }

    None
}

