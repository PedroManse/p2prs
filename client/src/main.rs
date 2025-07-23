use common::*;
use std::collections::HashMap;
use std::net::{SocketAddrV4, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;

mod file_server;
use file_server::FileServer;

use self::file_server::{FileSystem, SimpleFileSystem, FSRequest};

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

struct TrackerServerContext<FS: FileSystem> {
    peers: Peers,
    server: TcpStream,
    file_server: Arc<FileServer<FS>>,
}

impl<FS: FileSystem> TrackerServerContext<FS> {
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

    fn new(srv: TcpStream, fsrv: &Arc<FileServer<FS>>) -> Result<Self, std::io::Error> {
        let file_server = Arc::clone(&fsrv);
        srv.set_nonblocking(true)?;
        let mut slf = Self {
            peers: Peers::new(),
            server: srv,
            file_server,
        };
        let connect_msg = client::Message::Connect(client::Connect {
            serve_port: fsrv.server.local_addr()?.port(),
            file_list: slf.file_server.file_system.list_files(),
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
    let file_ctx = Arc::new(FileServer::<SimpleFileSystem>::new("127.0.0.1:4545".parse().unwrap())?);
    let mut track_ctx = TrackerServerContext::new(track_server, &file_ctx)?;

    std::thread::scope(|s| {
        loop {
            track_ctx.check_server_messages()?;
            if let Some(file_req) = file_ctx.check_serve()? {
                file_req.send_file_scoped_thread(s)?;
            };
        }
    })
}
