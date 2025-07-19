pub mod serial;
pub use serial::{FromBytes, IntoBytes};

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

impl From<client::Message> for AnyMessage {
    fn from(value: client::Message) -> Self {
        AnyMessage::Client(value)
    }
}

impl From<server::Message> for AnyMessage {
    fn from(value: server::Message) -> Self {
        AnyMessage::Server(value)
    }
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

    impl From<Connect> for Message {
        fn from(value: Connect) -> Self {
            Message::Connect(value)
        }
    }

    // 2. UpdateFiles
    #[derive(Debug)]
    pub struct UpdateFiles {
        pub file_list: Vec<File>,
    }

    impl From<UpdateFiles> for Message {
        fn from(value: UpdateFiles) -> Self {
            Message::UpdateFiles(value)
        }
    }

    // 3. Disconnect
    #[derive(Debug)]
    pub struct Disconnect;

    impl From<Disconnect> for Message {
        fn from(value: Disconnect) -> Self {
            Message::Disconnect(value)
        }
    }

    // 4. RequestFile
    #[derive(Debug)]
    pub struct RequestFile {
        pub file: PathBuf,
    }

    impl From<RequestFile> for Message {
        fn from(value: RequestFile) -> Self {
            Message::RequestFile(value)
        }
    }

    #[derive(Debug)]
    pub enum Message {
        Connect(Connect),
        UpdateFiles(UpdateFiles),
        Disconnect(Disconnect),
        RequestFile(RequestFile),
    }
}

// TODO Ipv4Addr -> SocketAddrV4
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

    impl From<RegisterPeer> for Message {
        fn from(value: RegisterPeer) -> Self {
            Message::RegisterPeer(value)
        }
    }

    // 2. UpdatePeer
    #[derive(Debug)]
    pub struct UpdatePeer {
        pub ip: Ipv4Addr,
        pub file_list: Vec<File>,
    }

    impl From<UpdatePeer> for Message {
        fn from(value: UpdatePeer) -> Self {
            Message::UpdatePeer(value)
        }
    }

    // 3. UnregisterPeer
    #[derive(Debug)]
    pub struct UnregisterPeer {
        pub ip: Ipv4Addr,
    }

    impl From<UnregisterPeer> for Message {
        fn from(value: UnregisterPeer) -> Self {
            Message::UnregisterPeer(value)
        }
    }

    #[derive(Debug)]
    pub enum Message {
        RegisterPeer(RegisterPeer),
        UpdatePeer(UpdatePeer),
        UnregisterPeer(UnregisterPeer),
    }
}
