use std::path::PathBuf;

use common::serial::{IntoBytes, FromRaw};
use common::*;

fn main() {
    let x = client::Message::RequestFile(client::RequestFile{file: PathBuf::from("file.txt")});
    let x_bytes = x.into_bytes();
    println!("{x_bytes:?}");
}
