use crate::{AnyMessage, MsgType, client, server};
use std::ffi::OsString;
use std::io::Read;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf; // for from_vec

pub trait FromBytes: Sized {
    type Error;
    fn from_stream(stream: &mut impl Read) -> Result<Self, Self::Error>;
}

macro_rules! impl_read {
    ( num $($t:ty)* ) => {
        $(
            impl FromBytes for $t {
                type Error = std::io::Error;
                fn from_stream(stream: &mut impl Read) -> Result<Self, Self::Error> {
                    let mut buf = [0u8; std::mem::size_of::<$t>()];
                    stream.read_exact(&mut buf)?;
                    Ok::<$t, Self::Error>(<$t>::from_le_bytes(buf))
                }
            }
        )*
    };

    ( [u8] => $convert:path => $bt:ty ) => {
        impl FromBytes for $bt {
            type Error = std::io::Error;
            fn from_stream(stream: &mut impl Read) -> Result<Self, Self::Error> {
                let bytes_size = u64::from_stream(stream)? as usize;
                let mut bytes = vec![0u8; bytes_size];
                stream.read_exact(&mut bytes)?;
                Ok($convert(bytes))
            }
        }
    };

    ( [u8] => $convert:expr => $bt:ty ) => {
        impl FromBytes for $bt {
            type Error = std::io::Error;
            fn from_stream(stream: &mut impl Read) -> Result<Self, Self::Error> {
                let bytes_size = u64::from_stream(stream)? as usize;
                let mut bytes = vec![0u8; bytes_size];
                stream.read_exact(&mut bytes)?;
                Ok($convert(bytes))
            }
        }
    };
}

impl_read!(num u8 u16 u32 u64 u128);
impl_read!(num i8 i16 i32 i64 i128);
impl_read!([u8] => |b|OsString::from_vec(b).into() => PathBuf );
impl_read!([u8] => OsString::from_vec => OsString);
