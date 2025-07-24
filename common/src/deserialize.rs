use crate::{AnyMessage, MsgType, client, server};
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt; // for from_vec
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct VecRead{
    buf: Vec<u8>,
    pointer: usize,
}

impl From<Vec<u8>> for VecRead {
    fn from(buf: Vec<u8>) -> Self {
        VecRead { buf, pointer: 0 }
    }
}

impl VecRead {
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DeserializeError> {
        let wish_take = buf.len();
        let new_pointer = self.pointer+wish_take;
        if new_pointer > self.buf.len() {
            Err(DeserializeError::EOF(self.clone(), buf.len()))
        } else {
            buf.copy_from_slice(&self.buf[self.pointer..new_pointer]);
            self.pointer = new_pointer;
            Ok(())
        }
    }
}

pub trait FromBytes: Sized {
    fn from_stream(stream: &mut VecRead) -> Result<Self, DeserializeError>;
}

macro_rules! impl_read {
    ( num $($t:ty)* ) => {
        $(
            impl FromBytes for $t {
                fn from_stream(stream: &mut VecRead) -> Result<Self, DeserializeError> {
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
            fn from_stream(stream: &mut VecRead) -> Result<Self, DeserializeError> {
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
            fn from_stream(stream: &mut VecRead) -> Result<Self, DeserializeError> {
                <$ot>::from_stream(stream).and_then($convert)
            }
        }
    };

    ( $ot:ty => $convert:expr => $rt:ty ) => {
        impl FromBytes for $rt {
            fn from_stream(stream: &mut VecRead) -> Result<Self, DeserializeError> {
                <$ot>::from_stream(stream).map($convert)
            }
        }
    };
}

impl_read!(num u8 u16 u32 u64 u128);
impl_read!(num i8 i16 i32 i64 i128);
impl_read!([u8] => OsString::from_vec => OsString);
impl_read!(OsString => PathBuf::from => PathBuf);
impl_read!(OsString => |s|s.into_string().map_err(DeserializeError::OsStringUTF8Error) => Result<String>);

#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
    #[error("Not enough bytes in stream {0:?} to read {1}")]
    EOF(VecRead, usize),
    #[error("Failed to convert {0:?} to a UTF-8 String")]
    OsStringUTF8Error(OsString),
}
