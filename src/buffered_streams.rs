use std::{
    io::{self, BufReader, BufWriter},
    net::TcpStream,
};

pub struct BufTcpStream {
    pub input: BufReader<TcpStream>,
    pub output: BufWriter<TcpStream>,
}

impl BufTcpStream {
    pub fn new(stream:TcpStream) -> io::Result<Self> {
        let input = BufReader::new(stream.try_clone()?);
        let output = BufWriter::new(stream);
        Ok(Self {input, output})
    }
}
