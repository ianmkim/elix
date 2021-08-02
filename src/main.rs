#![allow(dead_code)]
use std::collections::HashMap;

mod networking;
mod fileio;
mod buffered_streams;
use crate::networking::*;
use crate::fileio::*;
use crate::buffered_streams::*;

// cli arg parser
use clap::{Arg, App};

use std::net::{TcpStream, UdpSocket};
use std::io::Write;

fn main() {
    let mut app = App::new("Elix")
        .version("0.1.0")
        .author("Ian Kim <ian@ianmkim.com>")
        .about("A small, fast, and dirty file transfer utility")
        .subcommand(App::new("send")
            .about("Sends a file using Elix")
            .arg(Arg::new("filename")
                .index(1)
                .required(true)
                .about("A relative path to the file you want to send")))
        .subcommand(App::new("take")
            .about("Receive a file using Elix given a code")
            .arg(Arg::new("code")
                .index(1)
                .required(true)
                .about("A code that was either generated or given when sending a file")));

    let matches = app.clone().get_matches();

    if let Some(ref matches) = matches.subcommand_matches("send"){
        let filename = String::from(matches.value_of("filename").unwrap());
        // read file from io and store as byte vector
        let file:Vec<u8> = read_file_once(filename);

        /*
        let stream = search_for_peer().unwrap();
        let discovery_pair = tcp_to_addr(stream);
        println!("finished peer discovery");
        let res = send(discovery_pair, &"something".as_bytes().to_vec());
        */
        println!("listening for peer responses");
        listen_for_peer_response(file);

    } else if let Some(ref matches) = matches.subcommand_matches("take"){
        let _code = matches.value_of("code").unwrap();

        let stream = search_for_peer().unwrap();
        let discovery_pair = tcp_to_addr(stream);

        println!("waiting for bytes");
        let (bytes_length, buffer) = receive(discovery_pair);
        println!("received {:?} bytes", bytes_length);
    } else {
        app.print_long_help();
    }
}

// benchmarking code from here on out
mod benchmark;
use crate::benchmark::{bench_read_file_once, bench_read_file_chunk};
use std::time::Instant;

fn benchmark_file_loading() {
    let mut benchmarks: HashMap<&str, &dyn Fn() -> Vec<u8>> = HashMap::new();
    benchmarks.insert("read_file_once", &bench_read_file_once);
    benchmarks.insert("read_file_chunk", &bench_read_file_chunk);

    let mut prev:Vec<u8> = Vec::new();
    for (i, (k, bm)) in benchmarks.into_iter().enumerate() {
        let now = Instant::now();
        let res:Vec<u8> = bm();
        let elapsed = now.elapsed();
        if i != 0 {
            assert_eq!(res, prev);
        }
        prev = res;
        println!("{} took {:.2?}", k, elapsed);
    }
}

/*
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::thread;
use std::process;

static MULTI_CAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 1);
pub fn listen() {
  let socket_address: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 9778);
  let bind_addr = Ipv4Addr::new(0, 0, 0, 0);
  let socket = UdpSocket::bind(socket_address).unwrap();
  println!("Listening on: {}", socket.local_addr().unwrap());
  socket.join_multicast_v4(&MULTI_CAST_ADDR, &bind_addr).unwrap();
  loop {
      let mut buf = [0; 120];
      let (data, origin) = socket.recv_from(&mut buf).unwrap();
      let buf = &mut buf[..data];
      let message = String::from_utf8(buf.to_vec()).unwrap();
      println!("Server got: {} from {}", message, origin);
  }
}

pub fn cast() -> Result<(), std::io::Error> {
  let socket_address: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
  let socket = UdpSocket::bind(socket_address)?;
  socket.connect(SocketAddrV4::new(MULTI_CAST_ADDR, 9778))?;
  // Don't send messages to yourself.
  // In this case self discovery is for human developers, not machines.
  socket.set_multicast_loop_v4(false)?;
  let data = String::from("{\"username\": \"test\"}");
  loop {
    println!("Sent");
    socket.send(data.as_bytes())?;
    thread::sleep(std::time::Duration::new(2,0));
  }
  Ok(())
}

fn test() {
    thread::spawn(||{
        listen();
    });
    cast();
}
*/


