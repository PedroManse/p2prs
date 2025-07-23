use std::io::Write;
use crate::{client, server};

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

pub trait SerializeMessage {
    const MSG_TYPE: MsgType;
    fn size(&self) -> usize;
    fn write(&self, stream: impl Write) -> Result<(), std::io::Error>;
}

impl SerializeMessage for client::Disconnect {
    const MSG_TYPE: MsgType = MsgType::Disconnect;
    fn size(&self) -> usize {
        0
    }
    fn write(&self, _: impl Write) -> Result<(), std::io::Error> {
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
    fn write(&self, mut stream: impl Write) -> Result<(), std::io::Error> {
        // {serve_port}:u16 [ {file_size}:64 {path_len}:64 {path}:path_len ]*
        stream.write_all(&self.serve_port.to_le_bytes())?;
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
    fn write(&self, mut stream: impl Write) -> Result<(), std::io::Error> {
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
    fn write(&self, mut stream: impl Write) -> Result<(), std::io::Error> {
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
    fn write(&self, mut stream: impl Write) -> Result<(), std::io::Error> {
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
    fn write(&self, mut stream: impl Write) -> Result<(), std::io::Error> {
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
            std::mem::size_of::<u16>()
            + std::mem::size_of::<u32>()
        
    }
    fn write(&self, mut stream: impl Write) -> Result<(), std::io::Error> {
        // {serve_ip}:u32 {serve_port}:u16
        stream.write_all(&self.sock.ip().to_bits().to_le_bytes())?;
        stream.write_all(&self.sock.port().to_le_bytes())
    }
}

