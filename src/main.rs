mod networking;
mod network_utils;
mod bytes_util;
mod ui;

use crate::ui::build_arg_parser;
use std::net::{TcpListener};

use crate::networking::*;
use crate::network_utils::{
    tcp_to_addr,
    listen_for_peer_response,
    search_for_peer,
};

use log::info;
use std::io::Write;

fn main() {
    let start = std::time::Instant::now();
    // start logger with custom formatting to show time
    env_logger::Builder::from_default_env().format(move |buf, rec| {
        let t = start.elapsed().as_secs_f32();
        writeln!(buf, "{:.03} [{}] - {}", t, rec.level(), rec.args())
    }).init();


    let mut app = build_arg_parser("0.3.0");
    let matches = app.clone().get_matches();

    // send mode
    if let Some(ref matches) = matches.subcommand_matches("send"){
        let filename = String::from(matches.value_of("filename").unwrap());
        listen_for_peer_response(filename);
    } 

    // receive mode
    else if let Some(ref matches) = matches.subcommand_matches("take"){
        // get psuedorandom code from user
        let code = String::from(matches.value_of("code").unwrap());
        info!("Code from user: {:?}", code);
        // blocing operation, will only return 1) when it discovers a peer
        // AND 2) the peer has the correct code
        let mut listener = TcpListener::bind("0.0.0.0:0").unwrap();
        let stream = search_for_peer(code.clone(), listener).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        // start the blocking receiver
        match rt.block_on(receiver(code, tcp_to_addr(stream))) {
            Ok(_) => info!("Done."),
            Err(e) => info!("An Error Ocurred: {}", e),
        }
    } 

    else {
        app.print_long_help().unwrap();
    }
}
