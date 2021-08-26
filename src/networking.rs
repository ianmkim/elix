use std::net::{SocketAddr, TcpListener};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::io;

use tokio::task;
use tokio::net::TcpStream as AsyncTcpStream;
use tokio::net::TcpListener as AsyncTcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use futures::future::join_all;

extern crate crc32fast;
use crc32fast::Hasher;
use byteorder::{LittleEndian, WriteBytesExt};

use indicatif::{ProgressBar, ProgressStyle};

use std::fs::File;

use tokio::sync::Mutex;
use std::sync::{Arc};
use std::thread;

use crate::network_utils::{
    send_chunk_len,
    receive_chunk_len,
    send_file_name,
    receive_file_name,};

use crate::bytes_util::{
    pad_until_len,
    encode_usize_as_vec,
    decode_buffer_to_u32,
    decode_buffer_to_usize,
    get_chunk_len,};

use log::info;


type AddrPair = (SocketAddr, SocketAddr);
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync >>;
/// CAP is the chunk capacity, default is 256KBs
const CAP:usize = 1024 * 512;
const SPINNER_TEMPLATE:&str = "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} {binary_bytes_per_sec} ({eta})";

/// Given a code and a peer address, receives file chunks asynchronously
pub async fn receiver(_code: String, addrs:AddrPair, listener:TcpListener) -> Result<()>{
    info!("RUNNING RECEIVER");
    let addr = addrs.0;

    info!("{:?}, {:?}", addrs.0, addrs.1);

    // before any chunks are sent, metadata is received
    // let listener = TcpListener::bind(&addr).unwrap();
    info!("BEFORE RUNNING RECEIVE FILE NAME");
    let filename  = receive_file_name(&listener); // need to know to write to filesystem
    let chunk_len = receive_chunk_len(&listener); // need to know for progress and when to stop receiving
    drop(listener);
    info!("AFTER LISTENER DROP");

    let listener = AsyncTcpListener::bind(&addr).await?;
    info!("After ACPLISTENER");
    let mut futures = vec![];
    let mut chunks= 0;

    // download progress
    let mut downloaded = 0u64;
    let total_size = (chunk_len * CAP) as u64;

    println!("Receiving data from sender...");
    let pb = ProgressBar::new( total_size );
    pb.set_style(ProgressStyle::default_bar()
        .template(SPINNER_TEMPLATE)
        .progress_chars("#>-"));

    loop {
        // when new socket is opened, spawn a new receiving thread
        let (socket, _) = listener.accept().await?;
        let fut = tokio::spawn(receive_chunk(socket));

        // update progress
        downloaded += CAP as u64;
        pb.set_position(downloaded);

        futures.push(fut);
        chunks += 1;
        info!("Chunks {}", chunks);
        if chunks == chunk_len { break }
    }

    pb.finish_with_message("downloaded");

    // wait for all the threads to finish and sort the chunks based on their index
    info!("Joining all threads");
    let mut results = join_all(futures).await;
    info!("Sorting all fragments");
    // TODO replace with async write to improve speed
    results.sort_by_key(|k| k.as_ref().unwrap().as_ref().unwrap().0);

    info!("Writing data to filesystem ({})", filename);
    let f = File::create(filename).expect("Unable to create file");
    let mut f = BufWriter::new(f);
    let mut i = 0;
    let res_len = results.len();

    println!("\nWriting file to disk...");
    let pb = ProgressBar::new( (res_len * CAP) as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template(SPINNER_TEMPLATE)
        .progress_chars("=>-"));

    // write file sequentially because async IO is not worth the trouble
    for res in results {
        info!("{:02}% written", (i as f32/res_len as f32) * 100f32);
        i+=CAP;
        pb.set_position(i as u64);
        f.write_all(&res.as_ref().unwrap().as_ref().unwrap().1).expect("Unable to write data");
    }

    pb.finish_with_message("Written");
    Ok(())
}


/// Given filename, peer address, and thread limit, send files as chunks asynchronously
pub async fn sender(filename:String, addrs:AddrPair, thread_limit:usize) -> Result<()>{
    info!("RUNNING SENDER");
    let file = File::open(&filename).unwrap();
    let meta_data = file.metadata().unwrap();

    let mut reader = BufReader::with_capacity(CAP, file);
    let addr = addrs.1;

    let active_thread_count = Arc::new(Mutex::new(0u32));
    let mut futures = vec![];
    let mut frag_id = 0 as usize;

    send_file_name(filename, addr.clone());

    let chunk_len = get_chunk_len(meta_data, CAP);
    send_chunk_len(encode_usize_as_vec(chunk_len), addr.clone());

    let mut downloaded = 0u64;
    let total_size = (chunk_len * CAP )as u64;
    let pb = ProgressBar::new( total_size );
    pb.set_style(ProgressStyle::default_bar()
        .template(SPINNER_TEMPLATE)
        .progress_chars("#>-"));


    loop {
        // fill the buffer up to capacity
        let buffer = reader.fill_buf().unwrap().clone();
        let length = buffer.clone().len();
        if length == 0 { break }

        info!("Read {} bytes", length);
        let converted_buff = buffer.to_vec();
        info!("BUFFER VEC LEN {}", converted_buff.len());

        let counter = Arc::clone(&active_thread_count);
        // start thread to send chunk
        let fut = task::spawn(send(frag_id, addr.clone(), converted_buff, counter));

        downloaded += CAP as u64;
        pb.set_position(downloaded);

        frag_id += 1;
        futures.push(fut);
        reader.consume(length);


        // block until new room for threads become available
        // hotswapping is much faster than the waves appraoch
        // most linux systems have a socket num limit of 1024
        while *active_thread_count.lock().await >= thread_limit as u32 {
            info!("Number of active threads: {}", *active_thread_count.lock().await);
        }
        /*
        if futures.len() == thread_limit {
            let _results = join_all(futures).await;
            futures = Vec::new();
        }
        */
    }

    // join all the remaining sending threads
    let _results = join_all(futures).await;
    info!("After join all");

    Ok(())
}


/// Send one chunk
async fn send(frag_id:usize, addr:SocketAddr, mut bytes: Vec<u8>, counter:Arc<Mutex<u32>>) -> Result<(usize, bool)> {
    let mut num = counter.lock().await;
    *num += 1;
    drop(num);

    let mut stream = AsyncTcpStream::connect(addr).await.expect("Connection was closed unexpectedly");
    
    if bytes.len() != CAP {
        bytes = pad_until_len(bytes.clone(), CAP);
    }

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
            Ok(_) => {
                let received_sum = decode_buffer_to_u32(buffer);
                info!("Reply and sent equal? {:?}",  checksum == received_sum);
                if checksum != received_sum {
                    info!("Mismatch: {:?} | {:?}", checksum, received_sum);
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

    info!("Fragment send completely finished");

    let mut num = counter.lock().await;
    *num -= 1;
    drop(num);

    Ok((frag_id, not_corrupted))
}


/// Receive one chunk
async fn receive_chunk(mut socket:AsyncTcpStream) -> Result<(usize, Vec<u8>)>  {
    // first 4 bytes is for id and the other 4 bytes is to indicate length of
    // the following vector
    let mut comb_buf = vec![0;CAP + 4 + 4];
    loop {
        let n = socket
            .read_exact(&mut comb_buf)
            .await
            .expect("failed to read data from socket");

        if n == 0 {
            return Ok((usize::MAX, [0u8;0].to_vec()));
        }

        let id_bytes:Vec<_> = comb_buf.drain(0..4).collect();
        let id= decode_buffer_to_usize(id_bytes);
        info!("Fragment ID {}", id);

        let length_bytes:Vec<_> = comb_buf.drain(0..4).collect();
        let length = decode_buffer_to_usize(length_bytes);

        let mut buf:Vec<_> = comb_buf.drain(0..length).collect();
        info!("Fragment Length {}", buf.len());

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


