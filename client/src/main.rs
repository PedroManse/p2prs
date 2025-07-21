use common::*;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddrV4, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;

fn read_msg(stream: &mut std::net::TcpStream) -> AnyMessage {
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

    AnyMessage::from_header_and_content(msg_type, content_size, buf).unwrap()
}

#[derive(Debug)]
struct Peer {
    sock: SocketAddrV4,
    files: Vec<File>,
}

#[derive(Default)]
struct Peers {
    full: HashMap<SocketAddrV4, Arc<Peer>>,
}

impl Peers {
    fn new() -> Self {
        Self::default()
    }
    fn add_peer(&mut self, peer: Peer) -> Result<(), Arc<Peer>> {
        use std::collections::hash_map::Entry;
        let peer_rc = Arc::new(peer);
        match self.full.entry(peer_rc.sock.clone()) {
            Entry::Vacant(a) => a.insert(Arc::clone(&peer_rc)),
            Entry::Occupied(o) => Err(Arc::clone(o.get()))?,
        };
        Ok(())
    }
    fn update_peer(&mut self, peer: Peer) -> Result<(), ()> {
        use std::collections::hash_map::Entry;
        match self.full.entry(peer.sock.clone()) {
            Entry::Occupied(mut o) => o.insert(Arc::new(peer)),
            Entry::Vacant(..) => Err(())?,
        };
        Ok(())
    }
    fn remove_peer(&mut self, sock: SocketAddrV4) -> Option<Arc<Peer>> {
        self.full.remove(&sock)
    }
    fn get_peer(&mut self, sock: SocketAddrV4) -> Option<&Peer> {
        self.full.get(&sock).map(|v| v.as_ref())
    }
}

struct Context {
    peers: Peers,
    server: TcpStream,
    files: Vec<File>,
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

impl Context {
    fn handle_message(&mut self, msg: AnyMessage) {
        match msg {
            AnyMessage::Server(server::Message::RegisterPeer(p)) => {
                self.peers.add_peer(p.into()).unwrap();
            }
            AnyMessage::Server(server::Message::UpdatePeer(p)) => {
                self.peers.update_peer(p.into()).unwrap();
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

    fn new(srv: TcpStream) -> Self {
        let mut slf = Self {
            peers: Peers::new(),
            server: srv,
            files: vec![File {
                path: PathBuf::from("file.txt"),
                size: 30,
            }],
        };
        let connect_msg = client::Message::Connect(client::Connect {
            serve_port: 42069,
            file_list: slf.files.clone(),
        });
        write_msg(&mut slf.server, connect_msg).unwrap();
        slf
    }

    fn main_loop(&mut self) {
        loop {
            let m = read_msg(&mut self.server);
            println!("{m:?}");
            self.handle_message(m);
        }
    }

}

fn main() -> Result<(), std::io::Error> {
    let s = std::net::TcpStream::connect("127.0.0.1:6969").unwrap();
    let mut ctx = Context::new(s);
    ctx.main_loop();
    Ok(())
}
