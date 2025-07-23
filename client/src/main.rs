use common::serialize::SerializeMessage;
use common::*;
use std::io::{BufWriter, Write};
use std::net::SocketAddrV4;
use std::path::PathBuf;
use std::sync::Arc;

mod file_server;
use file_server::{FSRequest, FileServer};

mod tracker;
use tracker::TrackerServerContext;

fn main() -> Result<(), ClientError> {
    if std::env::args().nth(1) == Some("get".to_string()) {
        get_file_main()
    } else {
        serve_file_main()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Lib(#[from] CommonError),
    #[error(transparent)]
    AddrParseError(#[from] std::net::AddrParseError),
}

fn get_file_main() -> Result<(), ClientError> {
    use std::io::Read;
    let file_server_addr: SocketAddrV4 = "127.0.0.1:6969".parse().unwrap();

    let mut s = std::net::TcpStream::connect(file_server_addr).unwrap();
    let req_msg = client::Message::RequestFile(client::RequestFile {
        file: PathBuf::from("uwu.txt"),
    });
    write_msg(&mut s, req_msg)?;
    let mut buf = vec![0u8; 30];
    s.read(&mut buf)?;
    print!("{:?}", String::from_utf8(buf));
    Ok(())
}

fn serve_file_main() -> Result<(), ClientError> {
    let tracker_addr = "127.0.0.1:42069".parse()?;
    let file_server_addr = "127.0.0.1:6969".parse()?;

    let file_ctx = Arc::new(FileServer::<file_server::SimpleFileSystem>::new(
        file_server_addr,
    )?);
    let mut track_ctx = TrackerServerContext::new(tracker_addr, &file_ctx)?;

    std::thread::scope(|s| {
        loop {
            track_ctx.check_server_messages()?;
            if let Some(file_req) = file_ctx.check_serve()? {
                file_req.send_file_scoped_thread(s)?;
            };
        }
    })
}
