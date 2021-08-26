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
    pad_until_len,
};
use log::info;

use local_ip_address::local_ip;

type AddrPair = (SocketAddr, SocketAddr);

const CODE_SIZE:usize = 256;

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
    // whatever port is available since the peer discovery will retrieve
    // the port information as well
    let listener = TcpListener::bind(local_ip().unwrap().to_string() + ":0").unwrap();
    let socket = listener.local_addr().unwrap();
    info!("The Listener is below");
    info!("{:?}", listener.local_addr().unwrap());
    
    // generate unique code as a security measure
    let rand_string = generate_code();
    
    println!("To receive {} from another computer, run this command: \n\telix take {}", file,rand_string);
    thread::spawn(move || {
        // whenever a peer is discovered through UDP multicasting
        autodiscover_rs::run(&socket, Method::Broadcast("255.255.255.255:1337".parse::<SocketAddr>().unwrap()), |socket_addr| {
            loop {
                info!("{:?}", socket_addr.as_ref().unwrap());
                match TcpStream::connect(socket_addr.as_ref().unwrap()){
                    Ok(mut s) => {
                        let mut code_buf = [0u8; CODE_SIZE];
                        // block until you receive code from peer
                        loop { match s.read_exact(&mut code_buf){
                                Ok(_) => break,
                                Err(e) => info!("Error while reading buffer {:?}", e),}}
                        info!("Read the code");
                        // decode contents of the buffer to a code string
                        let decoded_code = decode_bytes_to_string(code_buf.to_vec());
                        // if the decoded code received from the peer is equal to
                        // code generated on this machine (sender)
                        if decoded_code == rand_string{
                            info!("Writing the ack byte");
                            // write an ack byte
                            s.write(&[1u8]).unwrap();
                            // start the blocking sender
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            match rt.block_on(sender(file.clone(), tcp_to_addr(s), 500)) {
                                // because there's no way to gracefully exit (unless I rewrite peer discovery)
                                // just exit once this one process finishes
                                Ok(_) => std::process::exit(0),
                                Err(e) => { info!("{:?}", e); std::process::exit(0); }
                            }
                        } else {s.write(&[0u8]).unwrap();}
                        break;
                    },
                    Err(_) => {}
                }
            }
        }).unwrap();
    });
    loop {}
}

/// Function searches for peers on the network using UDP multicasting
/// Returns an Option enums of a TcpStream
pub fn search_for_peer(code:String, mut listener:TcpListener) -> Option<TcpStream> {
    let socket = listener.local_addr().unwrap();
    info!("Local addr: {:?}", listener.local_addr().unwrap());

    // this thread will theoretically never end
    thread::spawn(move || {
        // when a peer is dicovered, do nothing because you have a seperate listener below
        autodiscover_rs::run(&socket, Method::Broadcast("255.255.255.255:1337".parse().unwrap()), |s| {
            info!("Discovered socket: {:?}", s.unwrap());
        }).unwrap();
    });

    // start receiving from receiver
    let mut incoming = listener.incoming();
    // encode the code as a byte vector
    let code_buf = pad_until_len(encode_string_as_bytes(code), CODE_SIZE);
    // block until connection received
    while let Some(stream) = incoming.next() {
        let mut stream = stream.unwrap();
        info!("Sender stream: {:?}", stream.peer_addr().unwrap());
        // send the code over to see if the current peer is the correct one
        stream.write(&code_buf).unwrap();
        info!("Sent the code");
        let mut resp_buf = [0u8;1];
        // block until acknowledgement received
        loop { match stream.read_exact(&mut resp_buf) { Ok(_)=>break, Err(e)=>info!("Error while reading {:?}", e), }}
        if resp_buf[0] == 1u8 { return Some(stream); }
    }
    None
}

/// Send the encoded filename
pub fn send_file_name(filename:String, addr:SocketAddr){
    // keep trying to connect until the receiver has set up the server
    loop {
        match TcpStream::connect(addr){
            Ok(mut stream) => {
                let encoded = encode_string_as_bytes(filename.clone());
                let padded_filename = pad_until_len(encoded, CODE_SIZE);
                stream.write(&padded_filename).unwrap();
                loop {
                    match stream.read_exact(&mut [0u8;4]){
                        Ok(_) => break,
                        Err(e) => info!("Error while filename: {:?}", e),
                    }
                }
                break;
            }, Err(_) => {}
        }
    }
}

/// Receive encoded filename
pub fn receive_file_name(listener:&TcpListener) -> String{
    let mut filename = String::new();
    for stream in listener.incoming() {
        info!("STREAM LOOP IN RECEIVE_FILE_NAME");
        let mut stream = stream.unwrap();
        let mut filename_buf = [0u8;CODE_SIZE];
        loop {
            println!("running the receive file name");
            match stream.read_exact(&mut filename_buf){
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

/// Send the total number of chunks
pub fn send_chunk_len(chunk_len:Vec<u8>, addr:SocketAddr){
    let mut stream = TcpStream::connect(addr).expect("Couldn't send the chunk length");
    stream.write(&chunk_len).unwrap();
    loop { match stream.read_exact(&mut [0u8;4]) {
        Ok(_) => break,
        Err(e) => info!("Error while sending chunk len: {:?}", e),
    }}
}

/// Receive the total number of chunks
pub fn receive_chunk_len(listener:&TcpListener) -> usize {
    let mut chunk_len = 0;
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut chunk_len_buf = [0u8;4];
        loop {
            match stream.read_exact(&mut chunk_len_buf) {
                Ok(_) => break,
                Err(e) => info!("Error while receiving chunk len: {:?}", e),
            }
        }
        chunk_len = decode_buffer_to_usize(chunk_len_buf.to_vec());
        info!("Chunk length {}", chunk_len);
        stream.write(&[0u8; 4]).unwrap();
        break
    }
    chunk_len
}

