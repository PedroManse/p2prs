use crate::{AnyMessage, client, server};

trait IntoRaw {
    fn into_raw(self) -> RawMessage;
}

pub trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
}

pub trait FromRaw: Sized {
    type Error;
    fn try_from_raw(raw: RawMessage) -> Result<Self, Self::Error>;
}

impl From<&AnyMessage> for MsgType {
    fn from(value: &AnyMessage) -> Self {
        match value {
            AnyMessage::Server(m) => MsgType::from(m),
            AnyMessage::Client(m) => MsgType::from(m),
        }
    }
}

impl From<&server::Message> for MsgType {
    fn from(value: &server::Message) -> Self {
        match value {
            server::Message::UpdatePeer(..) => MsgType::SUpdatePeer,
            server::Message::RegisterPeer(..) => MsgType::SRegisterPeer,
            server::Message::UnregisterPeer(..) => MsgType::SUnregisterPeer,
        }
    }
}

impl From<&client::Message> for MsgType {
    fn from(value: &client::Message) -> Self {
        match value {
            client::Message::Connect(..) => MsgType::CConnect,
            client::Message::Disconnect(..) => MsgType::CDisconnect,
            client::Message::UpdateFileListing(..) => MsgType::CUpdateFileListing,
            client::Message::RequestFile(..) => MsgType::CRequestFile,
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
enum MsgType {
    CConnect = 1,
    CUpdateFileListing = 3,
    CDisconnect = 5,
    CRequestFile = 7,
    SRegisterPeer = 9,
    SUpdatePeer = 11,
    SUnregisterPeer = 13,
}

#[derive(Debug)]
pub struct RawMessage {
    msg_type: MsgType,
    content: Vec<u8>,
}

const U64_BYTES: u64 = 8;
const UU64_BYTES: usize = 8;
const U32_BYTES: u64 = 4;
const UU32_BYTES: usize = 4;

// Client: 1
impl IntoBytes for client::Connect {
    fn into_bytes(self) -> Vec<u8> {
        // [ {file_size}:64 {path_len}:64 {path}:path_len ]*
        let size: u64 = self
            .file_list
            .iter()
            .map(|a| a.path.as_os_str().as_encoded_bytes().len() as u64 + U64_BYTES * 2)
            .sum();
        let mut content = vec![0; size as usize];
        let mut index = 0;
        for file in self.file_list {
            (content[index..index + UU64_BYTES]).copy_from_slice(&file.size.to_le_bytes());
            index += UU64_BYTES;
            let path_size = file.path.as_os_str().as_encoded_bytes().len();
            (content[index..index + UU64_BYTES]).copy_from_slice(&path_size.to_le_bytes());
            index += UU64_BYTES;
            (content[index..index + path_size])
                .copy_from_slice(file.path.as_os_str().as_encoded_bytes());
        }
        content
    }
}

// Client: 2
impl IntoBytes for client::UpdateFileListing {
    fn into_bytes(self) -> Vec<u8> {
        client::Connect {
            file_list: self.file_list,
        }
        .into_bytes()
    }
}

// Client: 3
impl IntoBytes for client::Disconnect {
    fn into_bytes(self) -> Vec<u8> {
        vec![]
    }
}

// Client: 4
impl IntoBytes for client::RequestFile {
    fn into_bytes(self) -> Vec<u8> {
        let path_bytes = self.file.as_os_str().as_encoded_bytes();
        let path_size = path_bytes.len();
        let mut content = vec![0; path_size + UU64_BYTES];
        content[0..UU64_BYTES].copy_from_slice(&path_size.to_le_bytes());
        content[UU64_BYTES..path_size+UU64_BYTES].copy_from_slice(path_bytes);
        content
    }
}

impl IntoRaw for client::Message {
    fn into_raw(self) -> RawMessage {
        let msg_type = MsgType::from(&self);
        let content = match self {
            Self::Connect(c) => c.into_bytes(),
            Self::UpdateFileListing(c) => c.into_bytes(),
            Self::RequestFile(c) => c.into_bytes(),
            Self::Disconnect(c) => c.into_bytes(),
        };
        RawMessage { msg_type, content }
    }
}

impl IntoBytes for server::RegisterPeer {
    fn into_bytes(self) -> Vec<u8> {
        // {ip}:u32 [ {file_size}:u64 {path_len}:u64 {path}:path_len ]*
        let files_size: u64 = self
            .file_list
            .iter()
            .map(|a| a.path.as_os_str().as_encoded_bytes().len() as u64 + U64_BYTES * 2)
            .sum();
        // +4 for ip
        let mut content = vec![0; files_size as usize + 4];
        let mut index = UU32_BYTES;
        content[0..UU32_BYTES].copy_from_slice(&self.ip.octets());
        for file in self.file_list {
            (content[index..index + UU64_BYTES]).copy_from_slice(&file.size.to_le_bytes());
            index += UU64_BYTES;
            let path_size = file.path.as_os_str().as_encoded_bytes().len();
            (content[index..index + UU64_BYTES]).copy_from_slice(&path_size.to_le_bytes());
            index += UU64_BYTES;
            (content[index..index + path_size])
                .copy_from_slice(file.path.as_os_str().as_encoded_bytes());
        }
        content
    }
}

impl IntoBytes for server::UpdatePeer {
    fn into_bytes(self) -> Vec<u8> {
        server::RegisterPeer {
            ip: self.ip,
            file_list: self.file_list,
        }
        .into_bytes()
    }
}

impl IntoBytes for server::UnregisterPeer {
    fn into_bytes(self) -> Vec<u8> {
        Vec::from(self.ip.octets())
    }
}

impl IntoRaw for server::Message {
    fn into_raw(self) -> RawMessage {
        let msg_type = MsgType::from(&self);
        let content = match self {
            Self::RegisterPeer(c) => c.into_bytes(),
            Self::UpdatePeer(c) => c.into_bytes(),
            Self::UnregisterPeer(c) => c.into_bytes(),
        };
        RawMessage { msg_type, content }
    }
}

impl IntoRaw for AnyMessage {
    fn into_raw(self) -> RawMessage {
        match self {
            Self::Client(c) => c.into_raw(),
            Self::Server(c) => c.into_raw(),
        }
    }
}

impl<M> IntoBytes for M
where
    M: IntoRaw,
{
    fn into_bytes(self) -> Vec<u8> {
        let raw = self.into_raw();
        println!("{raw:?}");
        // {type}:u8 {content size}:u64 {content}:size
        let mut content = vec![0; raw.content.len() + 9];
        content[0] = raw.msg_type as u8;
        (content[1..UU64_BYTES+1]).copy_from_slice(&raw.content.len().to_le_bytes());
        content[UU64_BYTES+1..].copy_from_slice(&raw.content);
        content
    }
}
