pub mod deserialize;
pub mod serialize;
pub use deserialize::{DeserializeError, read_msg};
use std::io::Write;
use std::net::TcpStream;

#[cfg(test)]
mod test;

#[derive(Debug)]
#[repr(u8)]
pub enum MsgType {
    Connect = 1,
    UpdateFiles = 2,
    Disconnect = 3,
    RequestFile = 4,
    RegisterPeer = 5,
    UpdatePeer = 6,
    UnregisterPeer = 7,
}

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub path: std::path::PathBuf,
    pub size: u64,
}

#[derive(Debug, PartialEq)]
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
    #[derive(Debug, PartialEq)]
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
    #[derive(Debug, PartialEq)]
    pub struct UpdateFiles {
        pub file_list: Vec<File>,
    }

    impl From<UpdateFiles> for Message {
        fn from(value: UpdateFiles) -> Self {
            Message::UpdateFiles(value)
        }
    }

    // 3. Disconnect
    #[derive(Debug, PartialEq)]
    pub struct Disconnect;

    impl From<Disconnect> for Message {
        fn from(value: Disconnect) -> Self {
            Message::Disconnect(value)
        }
    }

    // 4. RequestFile
    #[derive(Debug, PartialEq)]
    pub struct RequestFile {
        pub file: PathBuf,
    }

    impl From<RequestFile> for Message {
        fn from(value: RequestFile) -> Self {
            Message::RequestFile(value)
        }
    }

    #[derive(Debug, PartialEq)]
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
    #[derive(Debug, PartialEq)]
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
    #[derive(Debug, PartialEq)]
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
    #[derive(Debug, PartialEq)]
    pub struct UnregisterPeer {
        pub sock: SocketAddrV4,
    }

    impl From<UnregisterPeer> for Message {
        fn from(value: UnregisterPeer) -> Self {
            Message::UnregisterPeer(value)
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum Message {
        RegisterPeer(RegisterPeer),
        UpdatePeer(UpdatePeer),
        UnregisterPeer(UnregisterPeer),
    }
}

pub fn read_msg_nb(stream: &mut TcpStream) -> Result<Option<AnyMessage>, CommonError> {
    match read_msg(stream) {
        Ok(m) => Ok(Some(m)),
        Err(DeserializeError::IO(e)) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
        Err(e) => Err(CommonError::from(e)),
    }
}

pub fn write_msg(
    stream: &mut TcpStream,
    msg: &impl serialize::Serialize,
) -> Result<(), CommonError> {
    write_msg_d(stream, msg)
}

pub fn write_msg_d(
    stream: &mut impl Write,
    msg: &impl serialize::Serialize,
) -> Result<(), CommonError> {
    stream.write_all(&[msg.msg_type() as u8])?;
    stream.write_all(&u64::to_le_bytes(msg.size() as u64))?;
    msg.write(stream)?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum CommonError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Deserialize(#[from] DeserializeError),
}
