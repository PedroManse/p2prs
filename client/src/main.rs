use common::*;
use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddrV4, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone)]
struct Peer {
    sock: SocketAddrV4,
    files: Vec<File>,
}

#[derive(Default)]
struct Peers {
    full: HashMap<SocketAddrV4, Peer>,
}

impl Peers {
    fn new() -> Self {
        Self::default()
    }
    fn add_peer(&mut self, peer: Peer) -> Option<Peer> {
        self.full.insert(peer.sock.clone(), peer)
    }
    fn update_peer(&mut self, new_peer: Peer) {
        self.full.insert(new_peer.sock.clone(), new_peer);
    }
    fn remove_peer(&mut self, sock: SocketAddrV4) -> Option<Peer> {
        self.full.remove(&sock)
    }
    fn get_peer(&mut self, sock: SocketAddrV4) -> Option<&Peer> {
        self.full.get(&sock)
    }
}

impl From<server::RegisterPeer> for Peer {
    fn from(server::RegisterPeer { sock, file_list }: server::RegisterPeer) -> Self {
        Peer {
            sock,
            files: file_list,
        }
    }
}

impl From<server::UpdatePeer> for Peer {
    fn from(server::UpdatePeer { sock, file_list }: server::UpdatePeer) -> Self {
        Peer {
            sock,
            files: file_list,
        }
    }
}

struct TrackerServerContext {
    peers: Peers,
    server: TcpStream,
    files: Vec<File>,
}

impl TrackerServerContext {
    fn handle_message(&mut self, msg: AnyMessage) {
        match msg {
            AnyMessage::Server(server::Message::RegisterPeer(p)) => {
                self.peers.add_peer(p.into());
            }
            AnyMessage::Server(server::Message::UpdatePeer(p)) => {
                self.peers.update_peer(p.into());
            }
            AnyMessage::Server(server::Message::UnregisterPeer(p)) => {
                self.peers.remove_peer(p.sock).unwrap();
            }
            AnyMessage::Client(client::Message::RequestFile(f)) => {
                todo!("Serve file to {f:?}")
            }
            m => panic!("can't handle msg {m:?}"),
        }
    }

    fn new(srv: TcpStream, fsrv: &FileServer) -> Result<Self, std::io::Error> {
        srv.set_nonblocking(true)?;
        let mut slf = Self {
            peers: Peers::new(),
            server: srv,
            files: fsrv.files.clone(),
        };
        let connect_msg = client::Message::Connect(client::Connect {
            serve_port: fsrv.server.local_addr()?.port(),
            file_list: slf.files.clone(),
        });
        write_msg(&mut slf.server, connect_msg).unwrap();
        Ok(slf)
    }

    fn check_server_messages(&mut self) -> Result<(), std::io::Error> {
        if let Some(m) = read_msg_nb(&mut self.server)? {
            self.handle_message(m);
        }
        Ok(())
    }
}

struct FileServer {
    server: TcpListener,
    files: Vec<File>,
}

impl FileServer {
    fn new(addr: SocketAddrV4) -> Result<Self, std::io::Error> {
        let server = TcpListener::bind(addr)?;
        server.set_nonblocking(true)?;
        Ok(FileServer {
            server,
            files: vec![],
        })
    }
    fn add_file(&mut self, file: File) {
        self.files.push(file);
    }
    fn check_serve(&self) -> Result<Option<(TcpStream, PathBuf)>, std::io::Error> {
        match self.server.accept() {
            Ok((mut stream, _)) => {
                let m = read_msg(&mut stream);
                if let AnyMessage::Client(client::Message::RequestFile(f)) = m {
                    Ok(Some((stream, f.file)))
                } else {
                    eprintln!("{m:?}");
                    Ok(None)
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }
    fn send_file(&self, stream: &mut TcpStream, _: &Path) {
        stream.write(&[
            54, 57, 32, 54, 57, 32, 54, 57, 32, 54, 57, 32, 54, 57, 32, 54, 57, 32, 54, 57, 32, 54,
            57, 32, 54, 57, 32, 54, 57, 32,
        ]).unwrap();
    }
}

fn main() -> Result<(), std::io::Error> {
    if std::env::args().nth(1) == Some("get".to_string()) {
        get_file_main()
    } else {
        serve_file_main()
    }
}

fn get_file_main() -> Result<(), std::io::Error> {
    use std::io::Read;
    let mut s = std::net::TcpStream::connect("127.0.0.1:42065").unwrap();
    let req_msg = client::Message::RequestFile(client::RequestFile {
        file: PathBuf::from("uwu.txt"),
    });
    write_msg(&mut s, req_msg)?;
    let mut buf = vec![0u8; 30];
    s.read(&mut buf)?;
    print!("{:?}", String::from_utf8(buf));
    Ok(())
}

fn serve_file_main() -> Result<(), std::io::Error> {
    let track_server = std::net::TcpStream::connect("127.0.0.1:6969").unwrap();
    let fl_srv_addr: SocketAddrV4 = "127.0.0.1:42064".parse().unwrap();
    let mut file_ctx = FileServer::new(fl_srv_addr)?;
    file_ctx.add_file(File {
        path: PathBuf::from("uwu.txt"),
        size: 30,
    });
    let file_ctx = Arc::new(file_ctx);
    let mut track_ctx = TrackerServerContext::new(track_server, &file_ctx)?;

    std::thread::scope(|s| {
        loop {
            track_ctx.check_server_messages()?;
            if let Some((mut stream, path)) = file_ctx.check_serve()? {
                let fl = Arc::clone(&file_ctx);
                std::thread::Builder::new()
                    .name("Client/ServeFile".to_string())
                    .spawn_scoped(s, move || {
                        fl.send_file(&mut stream, &path);
                    })?;
            }
        }
    })
}
