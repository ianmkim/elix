#![allow(dead_code)]
use std::collections::HashMap;

mod networking;
mod fileio;
use crate::networking::*;
use crate::fileio::*;

// cli arg parser
use clap::{Arg, App};

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
    } else if let Some(ref matches) = matches.subcommand_matches("take"){
        let _code = matches.value_of("code").unwrap();
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
