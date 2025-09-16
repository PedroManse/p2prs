use common::*;
use std::io::Write;
use std::net::{SocketAddrV4, TcpListener, TcpStream};
use std::path::PathBuf;
use std::thread::{Scope, ScopedJoinHandle};

impl<FS: FileSystem> FileServer<FS> {
    pub fn new(addr: SocketAddrV4) -> Result<Self, std::io::Error> {
        let server = TcpListener::bind(addr)?;
        server.set_nonblocking(true)?;
        Ok(Self {
            server,
            file_system: FS::new(),
        })
    }
    pub fn check_serve(&self) -> Option<Result<FS::FileRecord<'_>, CommonError>> {
        match self.server.accept() {
            Ok((mut stream, _)) => match read_msg(&mut stream) {
                Ok(AnyMessage::Client(client::Message::RequestFile(f))) => {
                    Some(Ok(self.file_system.make_request(stream, f.file)))
                }
                Ok(..) => None,
                Err(e) => Some(Err(CommonError::Deserialize(e))),
            },
            Err(e) => Some(Err(CommonError::IO(e))),
        }
    }
}

pub trait FSRequest<'srv, FS>: Sized + 'srv {
    fn send_file(self);

    fn send_file_scoped_thread<'env, 'scope>(
        self,
        s: &'scope Scope<'scope, 'env>,
    ) -> Result<ScopedJoinHandle<'scope, ()>, std::io::Error>
    where
        'srv: 'scope,
        Self: Send,
        FS: Sync,
    {
        std::thread::Builder::new()
            .name("Client/ServeFile".to_string())
            .spawn_scoped(s, move || {
                self.send_file();
            })
    }
}

pub trait FileSystem: Sized {
    type FileRecord<'s>: FSRequest<'s, Self>
    where
        Self: 's;
    fn new() -> Self;
    fn list_files(&self) -> Vec<File>;
    fn make_request<'s>(&self, stream: TcpStream, path: PathBuf) -> Self::FileRecord<'s>;
}

pub struct FileServer<FS: FileSystem> {
    pub server: TcpListener,
    pub file_system: FS,
}

//pub struct FileRequest<'s, FS: FileSystem> {
//    server: &'s FileServer<FS>,
//    stream: TcpStream,
//    path: PathBuf,
//}

pub struct SimpleFileSystem {
    files: Vec<File>,
}
pub struct SimpleFileRequest {
    stream: TcpStream,
    path: PathBuf,
}

impl FSRequest<'_, SimpleFileSystem> for SimpleFileRequest {
    fn send_file(mut self) {
        let str = std::fs::read_to_string(self.path).unwrap();
        self.stream.write_all(str.as_bytes()).unwrap();
    }
}

impl FileSystem for SimpleFileSystem {
    type FileRecord<'s> = SimpleFileRequest;
    fn new() -> Self {
        Self {
            files: vec![File {
                path: PathBuf::from("hi.txt"),
                size: 3,
            }],
        }
    }
    fn list_files(&self) -> Vec<File> {
        self.files.clone()
    }
    fn make_request<'s>(&self, stream: TcpStream, path: PathBuf) -> Self::FileRecord<'s> {
        SimpleFileRequest { stream, path }
    }
}
