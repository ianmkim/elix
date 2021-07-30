#![allow(dead_code)]
use std::collections::HashMap;

mod networking;
use crate::networking::*;

fn main() {
    let res = search_for_peer();
    println!("Received connection from main {:?}", res.unwrap().peer_addr());
}

// benchmarking code from here on out
mod benchmark;
use crate::benchmark::{read_file_once, read_file_chunk};
use std::time::Instant;

fn benchmark_file_loading() {
    let mut benchmarks: HashMap<&str, &dyn Fn() -> Vec<u8>> = HashMap::new();
    benchmarks.insert("read_file_once", &read_file_once);
    benchmarks.insert("read_file_chunk", &read_file_chunk);

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
