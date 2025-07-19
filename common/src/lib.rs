pub mod serial;
pub use serial::{IntoBytes, FromRaw};

#[derive(Debug)]
pub struct File {
    pub path: std::path::PathBuf,
    pub size: u64,
}

#[derive(Debug)]
pub enum AnyMessage {
    Client(client::Message),
    Server(server::Message),
}

/// Messages a client can send
pub mod client {
    use super::File;
    use std::path::PathBuf;

    // 1. Connect
    #[derive(Debug)]
    pub struct Connect {
        pub file_list: Vec<File>,
    }

    // 2. UpdateFileListing
    #[derive(Debug)]
    pub struct UpdateFileListing {
        pub file_list: Vec<File>,
    }

    // 3. Disconnect
    #[derive(Debug)]
    pub struct Disconnect;

    // 4. RequestFile
    #[derive(Debug)]
    pub struct RequestFile {
        pub file: PathBuf,
    }

    #[derive(Debug)]
    pub enum Message {
        Connect(Connect),
        UpdateFileListing(UpdateFileListing),
        Disconnect(Disconnect),
        RequestFile(RequestFile),
    }
}

/// Messages a server can send
pub mod server {
    use super::File;
    use std::net::Ipv4Addr;

    // 1. RegisterPeer
    #[derive(Debug)]
    pub struct RegisterPeer {
        pub ip: Ipv4Addr,
        pub file_list: Vec<File>,
    }

    // 2. UpdatePeer
    #[derive(Debug)]
    pub struct UpdatePeer {
        pub ip: Ipv4Addr,
        pub file_list: Vec<File>,
    }

    // 3. UnregisterPeer
    #[derive(Debug)]
    pub struct UnregisterPeer {
        pub ip: Ipv4Addr,
    }

    #[derive(Debug)]
    pub enum Message {
        RegisterPeer(RegisterPeer),
        UpdatePeer(UpdatePeer),
        UnregisterPeer(UnregisterPeer),
    }
}
