use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};
use std::fs;
use std::collections::HashMap;

use std::io::Read;

fn main() {
    test();
}

// benchmarking code from here on out
mod benchmark;
use crate::benchmark::*;
use std::time::Instant;

fn test() {
    let mut benchmarks: HashMap<&str, &dyn Fn() -> _> = HashMap::new();
    benchmarks.insert("read_file_once", &read_file_once);
    benchmarks.insert("real_file_chunk", &read_file_chunk);

    for (k, bm) in benchmarks {
        let now = Instant::now();
        { bm(); }
        let elapsed = now.elapsed();
        println!("{} took {:.2?}", k, elapsed);
    }
}
