mod networking;
mod network_utils;
mod bytes_util;

use crate::networking::*;
use crate::network_utils::{
    tcp_to_addr,
    listen_for_peer_response,
    search_for_peer,
};

// cli arg parser
use clap::{Arg, App};
use log::info;

use std::io::Write;

fn main() {
    let start = std::time::Instant::now();
    env_logger::Builder::from_default_env().format(move |buf, rec| {
        let t = start.elapsed().as_secs_f32();
        writeln!(buf, "{:.03} [{}] - {}", t, rec.level(), rec.args())
    }).init();


    let mut app = App::new("Elix")
        .version("0.2.0")
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
    } 

    else if let Some(ref matches) = matches.subcommand_matches("take"){
        let code = String::from(matches.value_of("code").unwrap());
        let stream = search_for_peer(code.clone()).unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(receiver(code, tcp_to_addr(stream))) {
            Ok(_) => info!("Done."),
            Err(e) => info!("An Error Ocurred: {}", e),
        }
    } 

    else {
        app.print_long_help().unwrap();
    }
}
