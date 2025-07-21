use common::{AnyMessage, File, client, read_msg, server, write_msg};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Peer {
    pub serve_port: u16,
    pub files: Vec<File>,
    pub conn: Arc<Mutex<TcpStream>>,
}


fn handle(ctx: Arc<Mutex<Context>>, mut stream: TcpStream) {
    let m = read_msg(&mut stream);
    if let AnyMessage::Client(client::Message::Connect(client::Connect {
        file_list,
        serve_port,
    })) = m
    {
        let new_peer = Peer {
            serve_port,
            files: file_list,
            conn: Arc::new(Mutex::new(stream)),
        };
        ctx.lock().unwrap().register_peer(new_peer);
    } else {
        println!("{m:?}");
    }
    println!("{ctx:?}");
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
        let mut sock = {
            match new_peer.conn.lock().unwrap().peer_addr().unwrap() {
                std::net::SocketAddr::V4(v4) => v4,
                std::net::SocketAddr::V6(_) => panic!(""),
            }
        };
        sock.set_port(new_peer.serve_port);
        for peer in &self.peers {
            let msg = server::Message::RegisterPeer(server::RegisterPeer {
                sock,
                file_list: new_peer.files.clone(),
            });
            write_msg(&mut peer.conn.lock().unwrap(), msg).unwrap();
        }
        self.peers.push(new_peer);
    }
}

fn main() -> Result<(), std::io::Error> {
    let ctx = Arc::new(Mutex::new(Context::new()));
    let listener = TcpListener::bind("127.0.0.1:6969")?;
    for stream in listener.incoming() {
        let stream = stream?;
        let th_ctx = Arc::clone(&ctx);
        std::thread::spawn(|| handle(th_ctx, stream));
    }
    unreachable!()
}
