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
    pub fn check_serve(&self) -> Result<Option<FS::FileRecord<'_>>, CommonError> {
        match self.server.accept() {
            Ok((mut stream, _)) => {
                let m = read_msg(&mut stream)?;
                if let AnyMessage::Client(client::Message::RequestFile(f)) = m {
                    Ok(Some(self.file_system.make_request(stream, f.file)))
                } else {
                    eprintln!("{m:?}");
                    Ok(None)
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e)?,
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

impl<'s> FSRequest<'s, SimpleFileSystem> for SimpleFileRequest {
    fn send_file(mut self) {
        let str = std::fs::read_to_string(self.path).unwrap();
        self.stream.write(str.as_bytes()).unwrap();
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
