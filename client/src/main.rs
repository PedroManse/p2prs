use common::*;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;

fn main() {
    let x = client::Message::Connect(client::Connect {
        file_list: vec![File {
            path: PathBuf::from("file.txt"),
            size: 30,
        }],
    });
    let x = server::Message::UpdatePeer(server::UpdatePeer{
        sock: SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0x1110),
        file_list: vec![File {
            path: PathBuf::from("file.txt"),
            size: 30,
        }],
    });
    eprint!("{x:?} ->\n");

    let x_bytes = x.into_bytes();
    eprint!("{x_bytes:?} ->\n");

    let m = AnyMessage::from_bytes(&x_bytes);
    eprintln!("{m:?}");
}
