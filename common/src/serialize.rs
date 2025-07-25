use crate::{AnyMessage, MsgType, client, server};
use std::io::Write;

/// Creates the three seperate components
pub trait Serialize {
    fn msg_type(&self) -> MsgType;
    fn size(&self) -> usize;
    fn write(&self, stream: &mut impl Write) -> Result<(), std::io::Error>;
}

pub trait SerializeMessage {
    const MSG_TYPE: MsgType;
    fn msg_type(&self) -> MsgType {
        Self::MSG_TYPE
    }
    fn size(&self) -> usize;
    fn write(&self, stream: &mut impl Write) -> Result<(), std::io::Error>;
}

impl SerializeMessage for client::Disconnect {
    const MSG_TYPE: MsgType = MsgType::Disconnect;
    fn size(&self) -> usize {
        0
    }
    fn write(&self, _: &mut impl Write) -> Result<(), std::io::Error> {
        Ok(())
    }
}

impl SerializeMessage for client::Connect {
    const MSG_TYPE: MsgType = MsgType::Connect;
    fn size(&self) -> usize {
        self.file_list
            .iter()
            .map(|a| a.path.as_os_str().as_encoded_bytes().len() + std::mem::size_of::<u64>() * 2)
            .sum::<usize>()
            + std::mem::size_of::<u16>()
    }
    fn write(&self, stream: &mut impl Write) -> Result<(), std::io::Error> {
        // {serve_port}:u16 {file_count}:u32 [ {file_size}:64 {path_len}:64 {path}:path_len ]*
        stream.write_all(&self.serve_port.to_le_bytes())?;
        stream.write_all(&(self.file_list.len() as u32).to_le_bytes())?;
        for file in &self.file_list {
            stream.write_all(&file.size.to_le_bytes())?;
            stream.write_all(&file.path.as_os_str().as_encoded_bytes().len().to_le_bytes())?;
            stream.write_all(file.path.as_os_str().as_encoded_bytes())?;
        }
        Ok(())
    }
}

impl SerializeMessage for client::UpdateFiles {
    const MSG_TYPE: MsgType = MsgType::UpdateFiles;
    fn size(&self) -> usize {
        self.file_list
            .iter()
            .map(|a| a.path.as_os_str().as_encoded_bytes().len() + std::mem::size_of::<u64>() * 2)
            .sum()
    }
    fn write(&self, stream: &mut impl Write) -> Result<(), std::io::Error> {
        for file in &self.file_list {
            stream.write_all(&file.size.to_le_bytes())?;
            stream.write_all(&file.path.as_os_str().as_encoded_bytes().len().to_le_bytes())?;
            stream.write_all(file.path.as_os_str().as_encoded_bytes())?;
        }
        Ok(())
    }
}

impl SerializeMessage for client::RequestFile {
    const MSG_TYPE: MsgType = MsgType::RequestFile;
    fn size(&self) -> usize {
        self.file.as_os_str().as_encoded_bytes().len()
    }
    fn write(&self, stream: &mut impl Write) -> Result<(), std::io::Error> {
        stream.write_all(&self.file.as_os_str().as_encoded_bytes().len().to_le_bytes())?;
        stream.write_all(self.file.as_os_str().as_encoded_bytes())
    }
}

impl SerializeMessage for server::RegisterPeer {
    const MSG_TYPE: MsgType = MsgType::RegisterPeer;
    fn size(&self) -> usize {
        self.file_list
            .iter()
            .map(|a| a.path.as_os_str().as_encoded_bytes().len() + std::mem::size_of::<u64>() * 2)
            .sum::<usize>()
            + std::mem::size_of::<u16>()
            + std::mem::size_of::<u32>()
    }
    fn write(&self, stream: &mut impl Write) -> Result<(), std::io::Error> {
        // {serve_ip}:u32 {serve_port}:u16 [ {file_size}:64 {path_len}:64 {path}:path_len ]*
        stream.write_all(&self.sock.ip().to_bits().to_le_bytes())?;
        stream.write_all(&self.sock.port().to_le_bytes())?;
        for file in &self.file_list {
            stream.write_all(&file.size.to_le_bytes())?;
            stream.write_all(&file.path.as_os_str().as_encoded_bytes().len().to_le_bytes())?;
            stream.write_all(file.path.as_os_str().as_encoded_bytes())?;
        }
        Ok(())
    }
}

impl SerializeMessage for server::UpdatePeer {
    const MSG_TYPE: MsgType = MsgType::UpdatePeer;
    fn size(&self) -> usize {
        self.file_list
            .iter()
            .map(|a| a.path.as_os_str().as_encoded_bytes().len() + std::mem::size_of::<u64>() * 2)
            .sum::<usize>()
            + std::mem::size_of::<u16>()
            + std::mem::size_of::<u32>()
    }
    fn write(&self, stream: &mut impl Write) -> Result<(), std::io::Error> {
        // {serve_ip}:u32 {serve_port}:u16 [ {file_size}:64 {path_len}:64 {path}:path_len ]*
        stream.write_all(&self.sock.ip().to_bits().to_le_bytes())?;
        stream.write_all(&self.sock.port().to_le_bytes())?;
        for file in &self.file_list {
            stream.write_all(&file.size.to_le_bytes())?;
            stream.write_all(&file.path.as_os_str().as_encoded_bytes().len().to_le_bytes())?;
            stream.write_all(file.path.as_os_str().as_encoded_bytes())?;
        }
        Ok(())
    }
}

impl SerializeMessage for server::UnregisterPeer {
    const MSG_TYPE: MsgType = MsgType::UnregisterPeer;
    fn size(&self) -> usize {
        std::mem::size_of::<u16>() + std::mem::size_of::<u32>()
    }
    fn write(&self, stream: &mut impl Write) -> Result<(), std::io::Error> {
        // {serve_ip}:u32 {serve_port}:u16
        stream.write_all(&self.sock.ip().to_bits().to_le_bytes())?;
        stream.write_all(&self.sock.port().to_le_bytes())
    }
}

impl Serialize for client::Message {
    fn msg_type(&self) -> MsgType {
        match self {
            client::Message::Connect(m) => m.msg_type(),
            client::Message::Disconnect(m) => m.msg_type(),
            client::Message::UpdateFiles(m) => m.msg_type(),
            client::Message::RequestFile(m) => m.msg_type(),
        }
    }
    fn size(&self) -> usize {
        match self {
            client::Message::Connect(m) => m.size(),
            client::Message::Disconnect(m) => m.size(),
            client::Message::UpdateFiles(m) => m.size(),
            client::Message::RequestFile(m) => m.size(),
        }
    }
    fn write(&self, stream: &mut impl Write) -> Result<(), std::io::Error> {
        match self {
            client::Message::Connect(m) => m.write(stream),
            client::Message::Disconnect(m) => m.write(stream),
            client::Message::UpdateFiles(m) => m.write(stream),
            client::Message::RequestFile(m) => m.write(stream),
        }
    }
}

impl Serialize for server::Message {
    fn msg_type(&self) -> MsgType {
        match self {
            server::Message::UpdatePeer(m) => m.msg_type(),
            server::Message::RegisterPeer(m) => m.msg_type(),
            server::Message::UnregisterPeer(m) => m.msg_type(),
        }
    }
    fn size(&self) -> usize {
        match self {
            server::Message::UpdatePeer(m) => m.size(),
            server::Message::RegisterPeer(m) => m.size(),
            server::Message::UnregisterPeer(m) => m.size(),
        }
    }
    fn write(&self, stream: &mut impl Write) -> Result<(), std::io::Error> {
        match self {
            server::Message::UpdatePeer(m) => m.write(stream),
            server::Message::RegisterPeer(m) => m.write(stream),
            server::Message::UnregisterPeer(m) => m.write(stream),
        }
    }
}

impl Serialize for AnyMessage {
    fn size(&self) -> usize {
        match self {
            AnyMessage::Client(m) => m.size(),
            AnyMessage::Server(m) => m.size(),
        }
    }
    fn msg_type(&self) -> MsgType {
        match self {
            AnyMessage::Client(m) => m.msg_type(),
            AnyMessage::Server(m) => m.msg_type(),
        }
    }
    fn write(&self, stream: &mut impl Write) -> Result<(), std::io::Error> {
        match self {
            AnyMessage::Client(m) => m.write(stream),
            AnyMessage::Server(m) => m.write(stream),
        }
    }
}
