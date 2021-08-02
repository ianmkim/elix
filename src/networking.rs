use std::net;
use std::net::{TcpListener, TcpStream, UdpSocket, Ipv6Addr, IpAddr, SocketAddr, Shutdown};
use std::thread;
use std::io;
use std::env;
use std::io::{Read, Write};
use std::str::from_utf8;

use std::io::prelude::*;
use std::io::{BufWriter, BufReader};

use crate::buffered_streams::BufTcpStream;

// TODO benchmark autodiscover_rs and rewrite if necessary
use autodiscover_rs::{self, Method};

// used in serializing and deserializing primitive types to byte arrays
extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use std::mem;
use std::io::Cursor;

// for outputting progress to stdout
extern crate progress_streams;
use progress_streams::ProgressWriter;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use std::thread::JoinHandle;



/// Used only for debugging, prints the address of the peer
fn handle_client(stream: io::Result<TcpStream>, msg:&Vec<u8>) {
    let mut uw_stream = stream.unwrap();
    let discovery_pair = tcp_to_addr(uw_stream);
    let (local, peer) = discovery_pair.clone();
    println!("LOCAL {:?},  PEER {:?}", local, peer);
    
    while match TcpStream::connect(peer){
        Ok(_) => false,
        Err(_) => true,
    }{}

    let mut stream = TcpStream::connect(peer).expect("Connection was closed unexpectedly");

    let mut length_indicator = [0u8; mem::size_of::<u32>()];
    length_indicator.as_mut()
        .write_u32::<LittleEndian>(msg.len() as u32)
        .expect("Unable to write");
    stream.write(&length_indicator).unwrap();
    println!("Sending the length of vector");
    stream.write(&msg).unwrap();
    println!("Sent!");

    let mut handles:Vec<JoinHandle<()>> = Vec::new();

    for chunk in msg.chunks(1024 * 128 as usize) {
        let mut cloned = chunk.clone();
        /*
        handles.push(thread::spawn(|| {
            println!("Chunk size {} bytes",cloned.len());
        }));
        */
    }

    for handle in handles {
        handle.join();
    }
    println!("Finished");

}


pub fn listen_for_peer_response(file:Vec<u8>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let socket = listener.local_addr().unwrap();
    thread::spawn(move || {
        autodiscover_rs::run(&socket, Method::Broadcast("255.255.255.255:1337".parse::<SocketAddr>().unwrap()), |s| {
            handle_client(s, &file);
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
            //thread::spawn(|| handle_client(s, &vec![0u8;0]));
        }).unwrap();
    });

    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next() {
        return Some(stream.unwrap());
    }

    None
}

/// Given a TcpStream object return a string pair denoting local and peer address
pub fn tcp_to_addr(stream:TcpStream) -> (SocketAddr,SocketAddr) {
    let local = stream.local_addr().unwrap();
    let peer = stream.peer_addr().unwrap();
    (local,peer)
}

pub fn send(receiver:(SocketAddr, SocketAddr), msg:&'static Vec<u8>) -> usize {
    let (local, peer) = receiver;

    while match TcpStream::connect(peer){
        Ok(_) => false,
        Err(_) => true,
    }{}

    let mut stream = TcpStream::connect(peer).expect("Connection was closed unexpectedly");

    let mut length_indicator = [0u8; mem::size_of::<u32>()];
    length_indicator.as_mut()
        .write_u32::<LittleEndian>(msg.len() as u32)
        .expect("Unable to write");
    stream.write(&length_indicator).unwrap();
    println!("Sending the length of vector");
    stream.write(&msg).unwrap();
    println!("Sent!");

    let mut handles:Vec<JoinHandle<()>> = Vec::new();

    for chunk in msg.chunks(1024 * 128 as usize) {
        handles.push(thread::spawn(move || {
            println!("Chunk size {} bytes", chunk.len());
        }));
    }

    for handle in handles {
        handle.join();
    }
    println!("Finished");
    /*
    while match TcpStream::connect(peer) {
        Ok(mut stream) => {
            /*
            // usize to byte array without depending on external libraries
            let mut length_indicator = [0u8; mem::size_of::<u32>()];
            length_indicator.as_mut()
                .write_u32::<LittleEndian>(msg.len() as u32)
                .expect("Unable to write");
            stream.write(&length_indicator).unwrap();
            println!("Sent the length of vector");
            stream.write(&msg).unwrap();
            println!("Sent message");
            false
            */

            let handles:Vec<JoinHandle<()>> = msg_in_chunks.iter()
                .map(move |chunk| {
                    thread::spawn(move ||{
                        println!("chunk size {} bytes", chunk.len());
                    })
                })
                .collect();
            for handle in handles {
                handle.join();
            }
            false

        },
        Err(e) => {
            println!("Failed to connect: {}", e);
            true
        }
    }{}
    0 as usize
    */
    0 as usize
}

pub fn handle_received_client(mut stream:TcpStream){
    println!("receiving data");
    let mut size_buffer = [0u8; mem::size_of::<u32>()];
    let mut bufStream = BufTcpStream::new(stream).expect("Buffered Reader Creation Failed");

    bufStream.input.read(&mut size_buffer).unwrap();
    let mut cursor = Cursor::new(size_buffer);
    let size = cursor.read_u32::<LittleEndian>().unwrap() as usize;

    println!("{:?}", size);

    let mut buffer = vec![0; size];
    while match bufStream.input.read(&mut buffer) {
        Ok(size) => {
            println!("{:?}", buffer[0]);
            true
        }, Err(err) => {
            println!("An Error occurred");
            false
        }
    }{  }

    println!("{:?}", buffer);

    /*
    while match stream.read(&mut buffer){
        Ok(size) => {
            println!("Just received {}", buffer.len());
            stream.write(&buffer[0..size]).unwrap();
            true
        },
        Err(err) => {
            println!("An Error occurred");
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
    */
}

pub fn receive(receiver:(SocketAddr, SocketAddr)) -> (usize, Vec<u8>){
    let (local, peer) = receiver;
    println!("LOCAL {:?},  PEER {:?}", local, peer);
    let listener = TcpListener::bind(local).unwrap();
    println!("receiving connections");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let res = handle_received_client(stream);
            }
            Err(e) => {
                println!("Error");
            }
        }
    }

    (0, vec![0u8; 0])
}
