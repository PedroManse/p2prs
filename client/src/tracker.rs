use crate::ClientError;

use super::file_server::{FileServer, FileSystem};
use common::*;
use std::collections::HashMap;
use std::net::{SocketAddrV4, TcpStream};
use std::sync::Arc;

#[derive(Debug, Clone)]
struct Peer {
    sock: SocketAddrV4,
    files: Vec<File>,
}

#[derive(Default)]
pub struct Peers {
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

pub struct TrackerServerContext<FS: FileSystem> {
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

    pub fn new(srv: SocketAddrV4, fsrv: &Arc<FileServer<FS>>) -> Result<Self, ClientError> {
        let track_server = std::net::TcpStream::connect(srv).unwrap();
        track_server.set_nonblocking(true)?;

        let file_server = Arc::clone(&fsrv);
        let mut slf = Self {
            peers: Peers::new(),
            server: track_server,
            file_server,
        };
        let connect_msg = client::Message::Connect(client::Connect {
            serve_port: fsrv.server.local_addr()?.port(),
            file_list: slf.file_server.file_system.list_files(),
        });
        write_msg(&mut slf.server, &connect_msg).unwrap();
        Ok(slf)
    }

    pub fn check_server_messages(&mut self) -> Result<(), ClientError> {
        if let Some(m) = read_msg_nb(&mut self.server)? {
            self.handle_message(m);
        }
        Ok(())
    }
}
