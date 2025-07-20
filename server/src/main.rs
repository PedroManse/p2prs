use common::{AnyMessage, File, IntoBytes, client, server};
use std::io::{Read, Write};
use std::net::{SocketAddrV4, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct Peer {
    files: Vec<File>,
    conn: Arc<Mutex<TcpStream>>,
}

fn read_msg(stream: &mut TcpStream) -> AnyMessage {
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

fn handle(ctx: Arc<Mutex<Context>>, mut stream: TcpStream) {
    println!("{stream:?}");
    let m = read_msg(&mut stream);
    if let AnyMessage::Client(client::Message::Connect(client::Connect { file_list })) = m {
        let new_peer = Peer {
            files: file_list,
            conn: Arc::new(Mutex::new(stream)),
        };
        ctx.lock().unwrap().register_peer(new_peer);
    } else {
        println!("{m:?}");
    }
}

#[derive(Default)]
struct Context {
    peers: Vec<Peer>,
}

impl Context {
    fn new() -> Self {
        Self::default()
    }
    fn register_peer(&mut self, new_peer: Peer) {
        let sock = {
            match new_peer.conn.lock().unwrap().peer_addr().unwrap() {
                std::net::SocketAddr::V4(v4)=>v4,
                std::net::SocketAddr::V6(_)=>panic!(""),
            }
        };
        for peer in &self.peers {
            let buf = AnyMessage::from(server::Message::RegisterPeer(server::RegisterPeer {
                sock,
                file_list: new_peer.files.clone(),
            }));
            peer.conn.lock().unwrap().write(&buf.into_bytes()).unwrap();
        }
        self.peers.push(new_peer);
    }
}

fn main() -> Result<(), std::io::Error> {
    let ctx = Arc::new(Mutex::new(Context { peers: vec![] }));
    let listener = TcpListener::bind("127.0.0.1:6969")?;
    for stream in listener.incoming() {
        let stream = stream?;
        let th_ctx = Arc::clone(&ctx);
        std::thread::spawn(|| handle(th_ctx, stream));
    }
    unreachable!()
}
