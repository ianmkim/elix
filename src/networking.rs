use std::net::{TcpListener,TcpStream, SocketAddr, Shutdown};
use std::io::{Read, Write, BufRead, BufReader};
use std::io;

use autodiscover_rs::{self, Method};
use std::thread;
use std::thread::JoinHandle;

use tokio::task;
use tokio::net::TcpStream as AsyncTcpStream;
use tokio::net::TcpListener as AsyncTcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::io::AsyncWrite;
use futures::future::join_all;

extern crate crc32fast;
use crc32fast::Hasher;
use std::io::Cursor;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};

use std::{fs, fs::File, fs::Metadata};

use crate::network_utils::{
    tcp_to_addr,
    send_chunk_len,
    receive_chunk_len,};

use crate::bytes_util::{
    encode_usize_as_vec,
    decode_buffer_to_u32,
    decode_buffer_to_usize,
    get_chunk_len,};

use std::sync::{Arc, Mutex};

type AddrPair = (SocketAddr, SocketAddr);
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync >>;
const CAP:usize = 1024 * 16;



async fn send(frag_id:usize, addr:SocketAddr, bytes: Vec<u8>) -> Result<(usize, bool)> {
    let mut stream = AsyncTcpStream::connect(addr).await.expect("Connection was closed unexpectedly");

    let mut hasher = Hasher::new();
    hasher.update(bytes.clone().as_mut_slice());
    let checksum = hasher.finalize();

    let mut res_vec = encode_usize_as_vec(frag_id);
    res_vec.append(&mut encode_usize_as_vec(bytes.clone().len()));
    res_vec.append(&mut bytes.clone());

    stream.write_all(&res_vec.clone()).await?;

    let mut not_corrupted = false;
    loop {
        stream.readable().await?;
        let mut buffer = vec![0u8; 4];

        match stream.try_read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                let received_sum = decode_buffer_to_u32(buffer);
                println!("Reply and sent equal? {:?}",  checksum == received_sum);
                if checksum != received_sum {
                    println!("Mismatch: {:?} | {:?}", checksum, received_sum);
                } else { not_corrupted = true; }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    println!("Fragment send completely finished");
    Ok((frag_id, not_corrupted))
}



pub async fn sender(filename:String, addrs:AddrPair) -> Result<()>{
    let file = File::open(&filename).unwrap();
    let meta_data = file.metadata().unwrap();

    let mut reader = BufReader::with_capacity(CAP, file);
    let addr = addrs.1;

    let mut futures = vec![];
    let mut frag_id = 0 as usize;

    send_chunk_len(get_chunk_len(meta_data, CAP), addr.clone());

    loop {
        let buffer = reader.fill_buf().unwrap().clone();
        let length = buffer.clone().len();
        if length == 0 { break }

        println!("Read {} bytes", length);
        let fut = task::spawn(send(frag_id, addr.clone(), buffer.to_vec()));
        frag_id += 1;
        futures.push(fut);
        reader.consume(length);
    }

    let results = join_all(futures).await;
    println!("After join all");

    Ok(())
}


async fn receive_chunk(mut socket:AsyncTcpStream) -> Result<(usize, Vec<u8>)>  {
    // first 4 bytes is for id and the other 4 bytes is to indicate length of
    // the following vector
    let mut comb_buf = vec![0;1024*16 + 4 + 4];
    loop {
        let n = socket
            .read(&mut comb_buf)
            .await
            .expect("failed to read data from socket");

        if n == 0 {
            return Ok((usize::MAX, [0u8;0].to_vec()));
        }

        let id_bytes:Vec<_> = comb_buf.drain(0..4).collect();
        let id= decode_buffer_to_usize(id_bytes);
        println!("Fragment ID {}", id);

        let length_bytes:Vec<_> = comb_buf.drain(0..4).collect();
        let length = decode_buffer_to_usize(length_bytes);

        let mut buf:Vec<_> = comb_buf.drain(0..length).collect();
        println!("Fragment Length {}", buf.len());

        let mut hasher = Hasher::new();
        hasher.update(&mut buf);
        let checksum = hasher.finalize();
        let mut checksum_bytes = [0u8; 4];

        checksum_bytes.as_mut()
            .write_u32::<LittleEndian>(checksum)
            .expect("Unable to convert checksum to bytes");

        socket.write_all(&checksum_bytes)
            .await
            .expect("Failed to write checksum to socket");

        return Ok((id, buf));
    }

}

pub async fn receiver(code: String, addrs:AddrPair) -> Result<()>{
    let addr = addrs.0;

    let chunk_len = receive_chunk_len(addr);
    let listener = AsyncTcpListener::bind(&addr).await?;

    let mut futures = vec![];
    let mut chunks= 0;

    loop {
        let (mut socket, _) = listener.accept().await?;
        let fut = tokio::spawn(receive_chunk(socket));
        futures.push(fut);
        chunks += 1;
        println!("Chunks {}", chunks);
        if chunks == chunk_len { break }
    }

    let results = join_all(futures).await;
    println!("after joinall");
    for res in results {
        println!("{:?}", res);
    }

    Ok(())
}

