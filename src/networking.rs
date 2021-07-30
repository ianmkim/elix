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

/*
use std::time::Duration;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::thread;

static MULTI_CAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 1);

pub fn listen() -> Result<(), std::io::Error>{
    let socket_address:SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0,0,0,0), 9778);
    let bind_addr = Ipv4Addr::new(0,0,0,0);
    let socket = UdpSocket::bind(socket_address)?;
    socket.join_multicast_v4(&MULTI_CAST_ADDR, &bind_addr)?;
    println!("Listening on: {}", socket.local_addr().unwrap());
    loop {
        println!("waiting for messages");
        let mut buf = [0; 120];
        let (data, origin) = socket.recv_from(&mut buf)?;
        let buf = &mut buf[..data];
        let message = String::from_utf8(buf.to_vec()).unwrap();
        println!("server got: {} from {}", message, origin);
    }
}

pub fn cast() -> Result<(), std::io::Error> {
    let socket_address: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0,0,0,0), 0);
    let socket = UdpSocket::bind(socket_address)?;
    socket.connect(SocketAddrV4::new(MULTI_CAST_ADDR,  9778))?;
    socket.set_multicast_loop_v4(false)?;
    let data = String::from("Hello From the Client");
    loop {
        socket.send(data.as_bytes())?;
        thread::sleep(Duration::from_secs(2));
        println!("Sending ping");
    }
    Ok(())
}
*/
