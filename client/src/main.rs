use common::*;
use std::net::SocketAddrV4;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
    let file_server_addr: SocketAddrV4 = "127.0.0.1:43625".parse().unwrap();

    let mut s = std::net::TcpStream::connect(file_server_addr).unwrap();
    let req_msg = client::Message::RequestFile(client::RequestFile {
        file: PathBuf::from("uwu.txt"),
    });
    write_msg(&mut s, &req_msg)?;
    let mut buf = vec![0u8; 30];
    s.read(&mut buf)?;
    print!("{:?}", String::from_utf8(buf));
    Ok(())
}

fn serve_file_main() -> Result<(), ClientError> {
    let file_server_addr = "127.0.0.1:0".parse()?;
    let tracker_addr = "127.0.0.1:6969".parse()?;

    let file_ctx = Arc::new(FileServer::<file_server::SimpleFileSystem>::new(
        file_server_addr,
    )?);
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let track_ctx = Arc::new(Mutex::new(TrackerServerContext::new(
        tracker_addr,
        &file_ctx,
    )?));
    let track_ctx_th = Arc::clone(&track_ctx);

    let tracker_erros = tx.clone();
    std::thread::spawn(move || {
        loop {
            let mut track = match track_ctx_th.lock() {
                Ok(e) => e,
                Err(e) => {
                    tracker_erros.send(e.to_string()).unwrap();
                    return;
                }
            };
            if let Err(e) = track.check_server_messages() {
                tracker_erros.send(e.to_string()).unwrap();
            }
        }
    });

    let file_server_errors = tx.clone();
    std::thread::spawn(move || {
        loop {
            let fl_req = match file_ctx.check_serve() {
                Some(Ok(e)) => Some(e),
                Some(Err(e)) => {
                    file_server_errors.send(e.to_string()).unwrap();
                    None
                }
                None => None,
            };
            if let Some(file_req) = fl_req {
                std::thread::spawn(|| {
                    file_req.send_file();
                });
            }
        }
    });
    let error = rx.recv().unwrap();
    eprintln!("ERROR: {error}");
    Ok(())
}
