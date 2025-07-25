use crate::{AnyMessage, File, MsgType, client, server};
use std::ffi::OsString;
use std::io::Read;
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::os::unix::ffi::OsStringExt; // for from_vec
use std::path::PathBuf;

pub fn make_msg_type(m: u8) -> Result<MsgType, DeserializeError> {
    Ok(match m {
        1 => MsgType::Connect,
        2 => MsgType::UpdateFiles,
        3 => MsgType::Disconnect,
        4 => MsgType::RequestFile,
        5 => MsgType::RegisterPeer,
        6 => MsgType::UpdatePeer,
        7 => MsgType::UnregisterPeer,
        x => return Err(DeserializeError::WrongMsgType(x)),
    })
}

pub fn read_msg(stream: &mut TcpStream) -> Result<AnyMessage, DeserializeError> {
    use MsgType as M;
    use client::Message as C;
    use client::*;
    use server::Message as S;
    use server::*;

    let msg_type = u8::from_stream(stream).and_then(make_msg_type)?;
    let mut content = VecRead::from(Vec::from_stream(stream)?);

    Ok(match msg_type {
        M::Connect => C::from(Connect::from_stream(&mut content)?).into(),
        M::UpdateFiles => C::from(UpdateFiles::from_stream(&mut content)?).into(),
        M::Disconnect => C::from(Disconnect).into(),
        M::RequestFile => C::from(RequestFile::from_stream(&mut content)?).into(),
        M::RegisterPeer => S::from(RegisterPeer::from_stream(&mut content)?).into(),
        M::UpdatePeer => S::from(UpdatePeer::from_stream(&mut content)?).into(),
        M::UnregisterPeer => S::from(UnregisterPeer::from_stream(&mut content)?).into(),
    })
}

#[derive(Debug, Clone)]
pub struct VecRead {
    buf: Vec<u8>,
    pointer: usize,
}

impl From<Vec<u8>> for VecRead {
    fn from(buf: Vec<u8>) -> Self {
        VecRead { buf, pointer: 0 }
    }
}

impl Read for VecRead {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let can_take = self.buf.len() - self.pointer;
        let wish_take = buf.len();
        let will_take = std::cmp::min(can_take, wish_take);
        let new_pointer = self.pointer + will_take;
        buf.copy_from_slice(&self.buf[self.pointer..new_pointer]);
        self.pointer = new_pointer;
        Ok(will_take)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), std::io::Error> {
        let wish_take = buf.len();
        let new_pointer = self.pointer + wish_take;
        if new_pointer > self.buf.len() {
            eprintln!("{self:?}");
            Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "VecRead EOF Error",
            ))?
        } else {
            buf.copy_from_slice(&self.buf[self.pointer..new_pointer]);
            self.pointer = new_pointer;
            Ok(())
        }
    }
}

pub trait FromBytes: Sized {
    fn from_stream(stream: &mut impl Read) -> Result<Self, DeserializeError>;
}

macro_rules! impl_read {
    ( num $($t:ty)* ) => {
        $(
            impl FromBytes for $t {
                fn from_stream(stream: &mut impl Read) -> Result<Self, DeserializeError> {
                    let mut buf = [0u8; std::mem::size_of::<$t>()];
                    stream.read_exact(&mut buf)?;
                    Ok::<$t, DeserializeError>(<$t>::from_le_bytes(buf))
                }
            }
        )*
    };

    // Read byte array into $bt with $convert function or closure
    ( [u8] => $convert:expr => $bt:ty ) => {
        impl FromBytes for $bt {
            fn from_stream(stream: &mut impl Read) -> Result<Self, DeserializeError> {
                let bytes_size = u64::from_stream(stream)? as usize;
                let mut bytes = vec![0u8; bytes_size];
                stream.read_exact(&mut bytes)?;
                Ok($convert(bytes))
            }
        }
    };

    //// Read complex type $ot and $convert it into $rt
    ( $ot:ty => $convert:expr => Result<$rt:ty> ) => {
        impl FromBytes for $rt {
            fn from_stream(stream: &mut impl Read) -> Result<Self, DeserializeError> {
                <$ot>::from_stream(stream).and_then($convert)
            }
        }
    };

    ( $ot:ty => $convert:expr => $rt:ty ) => {
        impl FromBytes for $rt {
            fn from_stream(stream: &mut impl Read) -> Result<Self, DeserializeError> {
                <$ot>::from_stream(stream).map($convert)
            }
        }
    };
}

impl_read!(num u8 u16 u32 u64 u128);
impl_read!(num i8 i16 i32 i64 i128);
impl_read!([u8] => Vec::from => Vec<u8>);
impl_read!([u8] => OsString::from_vec => OsString);
impl_read!(OsString => PathBuf::from => PathBuf);
impl_read!(OsString => |s|s.into_string().map_err(DeserializeError::OsStringUTF8Error) => Result<String>);

impl FromBytes for client::Connect {
    fn from_stream(stream: &mut impl Read) -> Result<Self, DeserializeError> {
        // {serve_port}:u16 {file_count}:u32 [ {file_size}:u64 {path_len}:u64 {path}:path_len ]*
        let serve_port = u16::from_stream(stream)?;
        let file_count = u32::from_stream(stream)?;
        let mut file_list = Vec::with_capacity(file_count as usize);
        for _ in 0..file_count {
            let size = u64::from_stream(stream)?;
            let path = PathBuf::from_stream(stream)?;
            file_list.push(File { path, size });
        }
        Ok(Self {
            file_list,
            serve_port,
        })
    }
}

impl FromBytes for client::UpdateFiles {
    fn from_stream(stream: &mut impl Read) -> Result<Self, DeserializeError> {
        // {file_count}:u32 [ {file_size}:u64 {path_len}:u64 {path}:path_len ]*
        let file_count = u32::from_stream(stream)?;
        let mut file_list = Vec::with_capacity(file_count as usize);
        for _ in 0..file_count {
            let size = u64::from_stream(stream)?;
            let path = PathBuf::from_stream(stream)?;
            file_list.push(File { path, size });
        }
        Ok(Self { file_list })
    }
}

impl_read!(PathBuf => |file|client::RequestFile{file} => client::RequestFile);

impl FromBytes for server::RegisterPeer {
    fn from_stream(stream: &mut impl Read) -> Result<Self, DeserializeError> {
        // {serve_ip}:u32 {serve_port}:u16 {file_count}:u32 [ {file_size}:64 {path_len}:64 {path}:path_len ]*
        let ip = u32::from_stream(stream)?;
        let port = u16::from_stream(stream)?;
        let file_count = u32::from_stream(stream)?;
        let mut file_list = Vec::with_capacity(file_count as usize);
        for _ in 0..file_count {
            let size = u64::from_stream(stream)?;
            let path = PathBuf::from_stream(stream)?;
            file_list.push(File { path, size });
        }
        Ok(Self {
            sock: SocketAddrV4::new(Ipv4Addr::from_bits(ip), port),
            file_list,
        })
    }
}

impl_read!(server::RegisterPeer => |server::RegisterPeer{sock, file_list}|server::UpdatePeer{ sock, file_list  } => server::UpdatePeer);

impl FromBytes for server::UnregisterPeer {
    fn from_stream(stream: &mut impl Read) -> Result<Self, DeserializeError> {
        let ip = u32::from_stream(stream)?;
        let port = u16::from_stream(stream)?;
        Ok(Self {
            sock: SocketAddrV4::new(Ipv4Addr::from_bits(ip), port),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
    #[error("Not enough bytes in stream {0:?} to read {1}")]
    EOF(VecRead, usize),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Failed to convert {0:?} to a UTF-8 String")]
    OsStringUTF8Error(OsString),
    #[error("Failed to convert {0:?} to a Msg Type")]
    WrongMsgType(u8),
}
