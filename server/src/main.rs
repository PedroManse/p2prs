use std::io::Read;
use std::net::{TcpListener, TcpStream};

use common::{client, server, AnyMessage};

fn handle(mut stream: TcpStream) {
    println!("{stream:?}");
    let mut buf = vec![0; 9];
    stream.read(&mut buf).unwrap();

    // {type}:u8
    let msg_type = buf[0].try_into().unwrap();

    // {content size}:u64
    let mut content_size = [0u8; 8];
    content_size.copy_from_slice(&buf[1..8 + 1]);
    let content_size = u64::from_le_bytes(content_size);

    let mut buf = vec![0; content_size as usize];
    stream.read(&mut buf).unwrap();

    let m = AnyMessage::from_header_and_content(msg_type, content_size, buf);
    println!("{m:?}");
}

fn main() -> Result<(), std::io::Error>{
    let listener = TcpListener::bind("127.0.0.1:6969")?;
    for stream in listener.incoming() {
        handle(stream?);
    }
    Ok(())
}
