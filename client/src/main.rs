use common::*;
use std::path::PathBuf;

fn main() {
    let x = client::Message::Connect(client::Connect {
        file_list: vec![File {
            path: PathBuf::from("file.txt"),
            size: 30,
        }],
    });
    eprint!("{x:?} -> ");

    let x_bytes = x.into_bytes();
    eprint!("{x_bytes:?} -> ");

    let m = AnyMessage::from_bytes(&x_bytes);
    eprintln!("{m:?}");
}
