use crate::{AnyMessage, File, client, server};
use std::net::{Ipv4Addr, SocketAddrV4};

trait IntoRaw {
    fn into_raw(&self) -> RawMessage;
}

pub trait IntoBytes {
    fn into_bytes(&self) -> Vec<u8>;
}

pub trait FromBytes: Sized {
    type Error;
    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>;
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
            client::Message::UpdateFiles(..) => MsgType::CUpdateFiles,
            client::Message::RequestFile(..) => MsgType::CRequestFile,
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
enum MsgType {
    CConnect = 1,
    CUpdateFiles = 2,
    CDisconnect = 3,
    CRequestFile = 4,
    SRegisterPeer = 5,
    SUpdatePeer = 6,
    SUnregisterPeer = 7,
}

#[derive(Debug)]
pub struct RawMessage {
    msg_type: MsgType,
    content: Vec<u8>,
}

const U64_BYTES: usize = std::mem::size_of::<u64>();
const U32_BYTES: usize = std::mem::size_of::<u32>();
const U16_BYTES: usize = std::mem::size_of::<u16>() as usize;

// Client: 1
impl IntoBytes for client::Connect {
    fn into_bytes(&self) -> Vec<u8> {
        // [ {file_size}:64 {path_len}:64 {path}:path_len ]*
        let size: usize = self
            .file_list
            .iter()
            .map(|a| a.path.as_os_str().as_encoded_bytes().len() + U64_BYTES * 2)
            .sum();
        let mut content = vec![0; size as usize];
        let mut index = 0;
        for file in &self.file_list {
            (content[index..index + U64_BYTES]).copy_from_slice(&file.size.to_le_bytes());
            index += U64_BYTES;
            let path_size = file.path.as_os_str().as_encoded_bytes().len();
            (content[index..index + U64_BYTES]).copy_from_slice(&path_size.to_le_bytes());
            index += U64_BYTES;
            (content[index..index + path_size])
                .copy_from_slice(file.path.as_os_str().as_encoded_bytes());
        }
        content
    }
}

// Client: 2
impl IntoBytes for client::UpdateFiles {
    fn into_bytes(&self) -> Vec<u8> {
        // [ {file_size}:64 {path_len}:64 {path}:path_len ]*
        let size: usize = self
            .file_list
            .iter()
            .map(|a| a.path.as_os_str().as_encoded_bytes().len() + U64_BYTES * 2)
            .sum();
        let mut content = vec![0; size];
        let mut index = 0;
        for file in &self.file_list {
            (content[index..index + U64_BYTES]).copy_from_slice(&file.size.to_le_bytes());
            index += U64_BYTES;
            let path_size = file.path.as_os_str().as_encoded_bytes().len();
            (content[index..index + U64_BYTES]).copy_from_slice(&path_size.to_le_bytes());
            index += U64_BYTES;
            (content[index..index + path_size])
                .copy_from_slice(file.path.as_os_str().as_encoded_bytes());
        }
        content
    }
}

// Client: 3
impl IntoBytes for client::Disconnect {
    fn into_bytes(&self) -> Vec<u8> {
        vec![]
    }
}

// Client: 4
impl IntoBytes for client::RequestFile {
    fn into_bytes(&self) -> Vec<u8> {
        let path_bytes = self.file.as_os_str().as_encoded_bytes();
        let path_size = path_bytes.len();
        let mut content = vec![0; path_size + U64_BYTES];
        content[0..U64_BYTES].copy_from_slice(&path_size.to_le_bytes());
        content[U64_BYTES..path_size + U64_BYTES].copy_from_slice(path_bytes);
        content
    }
}

impl IntoRaw for client::Message {
    fn into_raw(&self) -> RawMessage {
        let msg_type = MsgType::from(self);
        let content = match self {
            Self::Connect(c) => c.into_bytes(),
            Self::UpdateFiles(c) => c.into_bytes(),
            Self::RequestFile(c) => c.into_bytes(),
            Self::Disconnect(c) => c.into_bytes(),
        };
        RawMessage { msg_type, content }
    }
}

impl IntoBytes for server::RegisterPeer {
    fn into_bytes(&self) -> Vec<u8> {
        // {ip}:u64 [ {file_size}:u64 {path_len}:u64 {path}:path_len ]*
        let files_size: usize = self
            .file_list
            .iter()
            .map(|a| a.path.as_os_str().as_encoded_bytes().len() + U64_BYTES * 2)
            .sum();
        // +4 for ip + 2 for port
        let mut content = vec![0; files_size + U32_BYTES + U16_BYTES];
        let mut index = U32_BYTES + size_of::<u16>();
        content[0..U32_BYTES].copy_from_slice(&self.sock.ip().octets());
        content[U32_BYTES..U32_BYTES + 2].copy_from_slice(&self.sock.port().to_le_bytes());
        for file in &self.file_list {
            (content[index..index + U64_BYTES]).copy_from_slice(&file.size.to_le_bytes());
            index += U64_BYTES;
            let path_size = file.path.as_os_str().as_encoded_bytes().len();
            (content[index..index + U64_BYTES]).copy_from_slice(&path_size.to_le_bytes());
            index += U64_BYTES;
            (content[index..index + path_size])
                .copy_from_slice(file.path.as_os_str().as_encoded_bytes());
        }
        content
    }
}

impl IntoBytes for server::UpdatePeer {
    fn into_bytes(&self) -> Vec<u8> {
        // {ip}:u32 [ {file_size}:u64 {path_len}:u64 {path}:path_len ]*
        let files_size: usize = self
            .file_list
            .iter()
            .map(|a| a.path.as_os_str().as_encoded_bytes().len() + U64_BYTES * 2)
            .sum();
        // +4 for ip + 2 for port
        let mut content = vec![0; files_size as usize + U32_BYTES + U16_BYTES];
        let mut index = U32_BYTES + U16_BYTES;
        content[0..U32_BYTES].copy_from_slice(&self.sock.ip().octets());
        content[U32_BYTES..U32_BYTES + U16_BYTES]
            .copy_from_slice(&self.sock.port().to_le_bytes());
        for file in &self.file_list {
            (content[index..index + U64_BYTES]).copy_from_slice(&file.size.to_le_bytes());
            index += U64_BYTES;
            let path_size = file.path.as_os_str().as_encoded_bytes().len();
            (content[index..index + U64_BYTES]).copy_from_slice(&path_size.to_le_bytes());
            index += U64_BYTES;
            (content[index..index + path_size])
                .copy_from_slice(file.path.as_os_str().as_encoded_bytes());
        }
        content
    }
}

impl IntoBytes for server::UnregisterPeer {
    fn into_bytes(&self) -> Vec<u8> {
        let mut content = vec![0u8; 6];
        content[0..U32_BYTES].copy_from_slice(&self.sock.ip().octets());
        content[U32_BYTES..U32_BYTES + U16_BYTES]
            .copy_from_slice(&self.sock.port().to_le_bytes());
        content
    }
}

impl IntoRaw for server::Message {
    fn into_raw(&self) -> RawMessage {
        let msg_type = MsgType::from(self);
        let content = match self {
            Self::RegisterPeer(c) => c.into_bytes(),
            Self::UpdatePeer(c) => c.into_bytes(),
            Self::UnregisterPeer(c) => c.into_bytes(),
        };
        RawMessage { msg_type, content }
    }
}

impl IntoRaw for AnyMessage {
    fn into_raw(&self) -> RawMessage {
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
    fn into_bytes(&self) -> Vec<u8> {
        let raw = self.into_raw();
        // {type}:u8 {content size}:u64 {content}:content size
        let mut content = vec![0; raw.content.len() + 9];
        content[0] = raw.msg_type as u8;
        (content[1..U64_BYTES + 1]).copy_from_slice(&raw.content.len().to_le_bytes());
        content[U64_BYTES + 1..].copy_from_slice(&raw.content);
        content
    }
}

impl TryFrom<u8> for MsgType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::CConnect,
            2 => Self::CUpdateFiles,
            3 => Self::CDisconnect,
            4 => Self::CRequestFile,
            5 => Self::SRegisterPeer,
            6 => Self::SUpdatePeer,
            7 => Self::SUnregisterPeer,
            x => panic!("Invalid msg type {x}"),
        })
    }
}

impl FromBytes for AnyMessage {
    type Error = ();
    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        let raw_msg = RawMessage::from_bytes(bytes)?;
        AnyMessage::try_from_raw(raw_msg)
    }
}

impl AnyMessage {
    pub fn from_header_and_content(msg_type: u8, content_size: u64, content: Vec<u8>) -> Result<Self, ()> {
        let msg_type = msg_type.try_into()?;
        assert!(content_size as usize == content.len());
        Ok(AnyMessage::try_from_raw(RawMessage { msg_type, content })?)
    }
}

impl FromBytes for RawMessage {
    type Error = ();
    fn from_bytes(bytes: &[u8]) -> Result<RawMessage, Self::Error> {
        // {type}:u8
        let msg_type = bytes[0].try_into()?;

        // {content size}:u64
        let mut content_size = [0u8; 8];
        content_size.copy_from_slice(&bytes[1..U64_BYTES + 1]);
        let content_size = u64::from_le_bytes(content_size);

        // {content}:content size
        let mut content = vec![0; content_size as usize];
        content.copy_from_slice(&bytes[1 + U64_BYTES..]);

        Ok(RawMessage { msg_type, content })
    }
}

impl FromRaw for AnyMessage {
    type Error = ();
    fn try_from_raw(raw: RawMessage) -> Result<Self, Self::Error> {
        use MsgType::*;
        use client::Message as CM;
        use server::Message as SM;
        Ok(match raw.msg_type {
            CConnect => CM::from(client::Connect::from_bytes(&raw.content)?).into(),
            CUpdateFiles => CM::from(client::UpdateFiles::from_bytes(&raw.content)?).into(),
            CDisconnect => CM::from(client::Disconnect).into(),
            CRequestFile => CM::from(client::RequestFile::from_bytes(&raw.content)?).into(),

            SRegisterPeer => SM::from(server::RegisterPeer::from_bytes(&raw.content)?).into(),
            SUpdatePeer => SM::from(server::UpdatePeer::from_bytes(&raw.content)?).into(),
            SUnregisterPeer => SM::from(server::UnregisterPeer::from_bytes(&raw.content)?).into(),
        })
    }
}

impl FromBytes for client::Connect {
    type Error = ();
    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        let x = client::UpdateFiles::from_bytes(bytes)?;
        Ok(client::Connect {
            file_list: x.file_list,
        })
    }
}

impl FromBytes for client::UpdateFiles {
    type Error = ();

    fn from_bytes(mut bytes: &[u8]) -> Result<Self, Self::Error> {
        use std::os::unix::ffi::OsStrExt; // for from_bytes
        let mut file_list = Vec::new();

        while !bytes.is_empty() {
            // Check if there are enough bytes for file_size and path_size
            if bytes.len() < 2 * U64_BYTES {
                panic!("Not enough bytes to read file metadata");
            }

            let mut file_size = [0u8; 8];
            file_size.copy_from_slice(&bytes[0..U64_BYTES]);
            let file_size = u64::from_le_bytes(file_size);

            let mut path_size = [0u8; 8];
            path_size.copy_from_slice(&bytes[U64_BYTES..U64_BYTES * 2]);
            let path_size = u64::from_le_bytes(path_size);

            let path_bytes = &bytes[2 * U64_BYTES..2 * U64_BYTES + path_size as usize];
            let os_str = std::ffi::OsStr::from_bytes(path_bytes);
            let path = std::path::PathBuf::from(os_str);

            file_list.push(crate::File {
                path,
                size: file_size,
            });

            // Move to the next file
            bytes = &bytes[2 * U64_BYTES + path_size as usize..];
        }

        Ok(client::UpdateFiles { file_list })
    }
}

impl FromBytes for client::RequestFile {
    type Error = ();

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        use std::os::unix::ffi::OsStrExt; // for from_bytes
        if bytes.len() < U64_BYTES {
            panic!("Byte slice too short to contain path size");
        }

        let mut size_bytes = [0u8; U64_BYTES];
        size_bytes.copy_from_slice(&bytes[0..U64_BYTES]);
        let path_size = u64::from_le_bytes(size_bytes) as usize;

        let path_bytes = &bytes[U64_BYTES..U64_BYTES + path_size];
        let os_str = std::ffi::OsStr::from_bytes(path_bytes);
        let path = std::path::PathBuf::from(os_str);

        Ok(client::RequestFile { file: path })
    }
}

impl FromBytes for server::RegisterPeer {
    type Error = ();
    fn from_bytes(mut bytes: &[u8]) -> Result<Self, Self::Error> {
        use std::os::unix::ffi::OsStrExt; // for from_bytes
        let mut ip = [0u8; 4];
        ip.copy_from_slice(&bytes[0..4]);
        let ip = Ipv4Addr::from_bits(u32::from_le_bytes(ip));
        let mut port = [0u8; 2];
        port.copy_from_slice(&bytes[4..6]);
        let port = u16::from_le_bytes(port);
        let sock = SocketAddrV4::new(ip, port);

        let mut file_list = vec![];

        bytes = &bytes[6..];
        while !bytes.is_empty() {
            let mut file_size = [0u8; U64_BYTES];
            file_size.copy_from_slice(&bytes[0..U64_BYTES]);
            let file_size = u64::from_le_bytes(file_size) as usize;

            let mut path_size = [0u8; U64_BYTES];
            path_size.copy_from_slice(&bytes[U64_BYTES..U64_BYTES * 2]);
            let path_size = u64::from_le_bytes(path_size) as usize;

            let os_str =
                std::ffi::OsStr::from_bytes(&bytes[U64_BYTES * 2..U64_BYTES * 2 + path_size]);
            let path = std::path::PathBuf::from(os_str);

            file_list.push(File {
                path,
                size: file_size as u64,
            });
            bytes = &bytes[U64_BYTES * 2 + path_size..];
        }

        Ok(server::RegisterPeer { file_list, sock })
    }
}

impl FromBytes for server::UpdatePeer {
    type Error = ();
    fn from_bytes(mut bytes: &[u8]) -> Result<Self, Self::Error> {
        use std::os::unix::ffi::OsStrExt; // for from_bytes
        let mut ip = [0u8; 4];
        ip.copy_from_slice(&bytes[0..4]);
        let ip = Ipv4Addr::from_bits(u32::from_le_bytes(ip));
        let mut port = [0u8; 2];
        port.copy_from_slice(&bytes[4..6]);
        let port = u16::from_le_bytes(port);
        let sock = SocketAddrV4::new(ip, port);

        let mut file_list = vec![];

        bytes = &bytes[6..];
        while !bytes.is_empty() {
            let mut file_size = [0u8; U64_BYTES];
            file_size.copy_from_slice(&bytes[0..U64_BYTES]);
            let file_size = u64::from_le_bytes(file_size) as usize;

            let mut path_size = [0u8; U64_BYTES];
            path_size.copy_from_slice(&bytes[U64_BYTES..U64_BYTES * 2]);
            let path_size = u64::from_le_bytes(path_size) as usize;

            let os_str =
                std::ffi::OsStr::from_bytes(&bytes[U64_BYTES * 2..U64_BYTES * 2 + path_size]);
            let path = std::path::PathBuf::from(os_str);

            file_list.push(File {
                path,
                size: file_size as u64,
            });
            bytes = &bytes[U64_BYTES * 2 + path_size..];
        }

        Ok(server::UpdatePeer { file_list, sock })
    }
}

impl FromBytes for server::UnregisterPeer {
    type Error = ();
    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut ip = [0u8; 4];
        ip.copy_from_slice(&bytes[0..4]);
        let ip = Ipv4Addr::from_bits(u32::from_le_bytes(ip));
        let mut port = [0u8; 2];
        port.copy_from_slice(&bytes[4..6]);
        let port = u16::from_le_bytes(port);
        let sock = SocketAddrV4::new(ip, port);
        Ok(server::UnregisterPeer{
            sock
        })
    }
}
