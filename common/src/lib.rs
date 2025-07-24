pub mod serial;
pub mod serialize;
pub use serial::FromBytes;
use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(Debug, Clone)]
pub struct File {
    pub path: std::path::PathBuf,
    pub size: u64,
}

#[derive(Debug)]
pub enum AnyMessage {
    Client(client::Message),
    Server(server::Message),
}

impl From<client::Message> for AnyMessage {
    fn from(value: client::Message) -> Self {
        AnyMessage::Client(value)
    }
}

impl From<server::Message> for AnyMessage {
    fn from(value: server::Message) -> Self {
        AnyMessage::Server(value)
    }
}

/// Messages a client can send
pub mod client {
    use super::File;
    use std::path::PathBuf;

    // 1. Connect
    #[derive(Debug)]
    pub struct Connect {
        pub file_list: Vec<File>,
        pub serve_port: u16,
    }

    impl From<Connect> for Message {
        fn from(value: Connect) -> Self {
            Message::Connect(value)
        }
    }

    // 2. UpdateFiles
    #[derive(Debug)]
    pub struct UpdateFiles {
        pub file_list: Vec<File>,
    }

    impl From<UpdateFiles> for Message {
        fn from(value: UpdateFiles) -> Self {
            Message::UpdateFiles(value)
        }
    }

    // 3. Disconnect
    #[derive(Debug)]
    pub struct Disconnect;

    impl From<Disconnect> for Message {
        fn from(value: Disconnect) -> Self {
            Message::Disconnect(value)
        }
    }

    // 4. RequestFile
    #[derive(Debug)]
    pub struct RequestFile {
        pub file: PathBuf,
    }

    impl From<RequestFile> for Message {
        fn from(value: RequestFile) -> Self {
            Message::RequestFile(value)
        }
    }

    #[derive(Debug)]
    pub enum Message {
        Connect(Connect),
        UpdateFiles(UpdateFiles),
        Disconnect(Disconnect),
        RequestFile(RequestFile),
    }
}

/// Messages a server can send
pub mod server {
    use super::File;
    use std::net::SocketAddrV4;

    // 1. RegisterPeer
    #[derive(Debug)]
    pub struct RegisterPeer {
        pub sock: SocketAddrV4,
        pub file_list: Vec<File>,
    }

    impl From<RegisterPeer> for Message {
        fn from(value: RegisterPeer) -> Self {
            Message::RegisterPeer(value)
        }
    }

    // 2. UpdatePeer
    #[derive(Debug)]
    pub struct UpdatePeer {
        pub sock: SocketAddrV4,
        pub file_list: Vec<File>,
    }

    impl From<UpdatePeer> for Message {
        fn from(value: UpdatePeer) -> Self {
            Message::UpdatePeer(value)
        }
    }

    // 3. UnregisterPeer
    #[derive(Debug)]
    pub struct UnregisterPeer {
        pub sock: SocketAddrV4,
    }

    impl From<UnregisterPeer> for Message {
        fn from(value: UnregisterPeer) -> Self {
            Message::UnregisterPeer(value)
        }
    }

    #[derive(Debug)]
    pub enum Message {
        RegisterPeer(RegisterPeer),
        UpdatePeer(UpdatePeer),
        UnregisterPeer(UnregisterPeer),
    }
}

#[deprecated]
pub fn read_msg(stream: &mut TcpStream) -> AnyMessage {
    let mut buf = vec![0; 9];
    stream.read(&mut buf).unwrap();
    println!("{buf:?}");

    // {type}:u8
    let msg_type = buf[0].try_into().unwrap();

    // {content size}:u64
    let mut content_size = [0u8; 8];
    content_size.copy_from_slice(&buf[1..8 + 1]);
    let content_size = u64::from_le_bytes(content_size);

    let mut buf = vec![0; content_size as usize];
    stream.read(&mut buf).unwrap();
    println!("{buf:?}");

    AnyMessage::from_header_and_content(msg_type, content_size, buf).unwrap()
}

pub fn read_msg_nb(stream: &mut TcpStream) -> Result<Option<AnyMessage>, CommonError> {
    match read_msg_nb_i(stream) {
        Ok(m) => Ok(Some(m)),
        Err(CommonError::IO(e)) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
        Err(e) => Err(e),
    }
}

fn read_msg_nb_i(stream: &mut TcpStream) -> Result<AnyMessage, CommonError> {
    let mut buf = vec![0; 9];
    stream.read(&mut buf)?;

    // {type}:u8
    let msg_type = buf[0];

    // {content size}:u64
    let mut content_size = [0u8; 8];
    content_size.copy_from_slice(&buf[1..8 + 1]);
    let content_size = u64::from_le_bytes(content_size);

    let mut buf = vec![0; content_size as usize];
    stream.read(&mut buf)?;

    let msg = AnyMessage::from_header_and_content(msg_type, content_size, buf).unwrap();
    Ok(msg)
}

pub fn write_msg(
    stream: &mut TcpStream,
    msg: &impl serialize::Serialize,
) -> Result<(), CommonError> {
    stream.write(&[msg.msg_type() as u8])?;
    stream.write(&u64::to_le_bytes(msg.size() as u64))?;
    msg.write(stream)?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum CommonError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
}
