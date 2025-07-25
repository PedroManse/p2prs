use common::{AnyMessage, CommonError, File, client, read_msg, server, write_msg};
use std::net::{SocketAddrV4, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Peer {
    pub server_addr: SocketAddrV4,
    pub files: Vec<File>,
    pub conn: Arc<Mutex<TcpStream>>,
}

#[allow(clippy::needless_pass_by_value)]
fn handle(ctx: Arc<Mutex<Context>>, mut stream: TcpStream) -> Result<(), CommonError> {
    let m = read_msg(&mut stream)?;
    println!("{m:?}");
    if let AnyMessage::Client(client::Message::Connect(client::Connect {
        file_list,
        serve_port,
    })) = m
    {
        let mut server_addr = match stream.peer_addr().unwrap() {
            std::net::SocketAddr::V4(v4) => v4,
            std::net::SocketAddr::V6(_) => panic!(""),
        };
        server_addr.set_port(serve_port);
        let new_peer = Peer {
            server_addr,
            files: file_list,
            conn: Arc::new(Mutex::new(stream)),
        };
        ctx.lock().unwrap().register_peer(new_peer);
    } else {
        println!("{m:?}");
    }
    Ok(())
}

#[derive(Default, Debug)]
struct Context {
    peers: Vec<Peer>,
}

impl Context {
    fn new() -> Self {
        Self::default()
    }
    fn register_peer(&mut self, new_peer: Peer) {
        let sock = new_peer.server_addr;
        let msg = server::Message::RegisterPeer(server::RegisterPeer {
            sock,
            file_list: new_peer.files.clone(),
        });
        self.peers.retain_mut(
            |peer| match write_msg(&mut peer.conn.lock().unwrap(), &msg) {
                Ok(..) => true,
                Err(common::CommonError::IO(e)) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                    false
                }
                Err(e) => panic!("{e}"),
            },
        );
        self.peers.iter().for_each(|p| {
            let msg = server::Message::RegisterPeer(server::RegisterPeer {
                sock: p.server_addr,
                file_list: new_peer.files.clone(),
            });
            write_msg(&mut new_peer.conn.lock().unwrap(), &msg).unwrap();
        });
        self.peers.push(new_peer);
    }
}

fn main() -> Result<(), CommonError> {
    let ctx = Arc::new(Mutex::new(Context::new()));
    let listener = TcpListener::bind("127.0.0.1:6969")?;
    for stream in listener.incoming() {
        println!("{stream:?}");
        let stream = stream?;
        let th_ctx = Arc::clone(&ctx);
        std::thread::Builder::new()
            .name("Server/HandlePeer".to_string())
            .spawn(|| handle(th_ctx, stream))?;
    }
    unreachable!()
}
