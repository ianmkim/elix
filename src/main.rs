#![allow(dead_code)]
use std::collections::HashMap;

mod networking;
use crate::networking::*;

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
        listen_for_peer_response(filename);
    } else if let Some(ref matches) = matches.subcommand_matches("take"){
        let code = String::from(matches.value_of("code").unwrap());
        let stream = search_for_peer().unwrap();
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(receiver(code, tcp_to_addr(stream))) {
            Ok(_) => println!("Done."),
            Err(e) => println!("An Error Ocurred: {}", e),
        }
    } else {
        app.print_long_help();
    }
}
