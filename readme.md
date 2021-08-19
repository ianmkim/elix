# Elix
Elix is a memory safe file transfer utility similar to croc written in pure Rust. 

<img src="https://github.com/parvusvox/elix/blob/master/docs/elix-logo.jpg?raw=true" width="300px">

### Installation
```
git clone https://github.com/parvusvox/elix
cd elix
cargo build
mv target/debug/elix /usr/bin
```

### Usage
Run this on a sender computer and remember the code it gives you
```
elix send <filename>
```
Then run this on the receiver computer
```
elix take <code>
```


## A small, fast, and dirty file transfer utility
Elix in latin translates to a drain or a ditch; oftentimes carrying shit. This naming denotes that elix is meant as a learning project and therefore is very unstable and prone to failure.

## Todos
 - [x] variable chunk size
 - [x] variable max socket opened
 - [x] create a new logo
 - [x] write documentation
 - [ ] create a lib version
 - [ ] add compression to chunks
 - [ ] add encryption to chunks
 - [ ] add archives through the -r or --recursive flag
 - [x] add speed viz in the cli
 - [ ] optimize socket thread join
