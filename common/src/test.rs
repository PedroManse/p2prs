use crate::*;
use std::path::PathBuf;

fn test_serialize_deserialize(m: AnyMessage) -> Result<(), CommonError> {
    let mut writer = Vec::new();
    write_msg_d(&mut writer, &m)?;
    let mut reader = crate::deserialize::VecRead::from(writer);
    let pm = read_msg(&mut reader)?;
    assert_eq!(m, pm);
    Ok(())
}

#[test]
fn test_de_serial() -> Result<(), CommonError> {
    let file = || File {
        path: PathBuf::from("hi.txt"),
        size: 1024 * 1024 * 4, // 4 MiB
    };
    let msgs: [AnyMessage; 7] = [
        client::Message::Connect(client::Connect {
            serve_port: 0,
            file_list: vec![file(), file(), file()],
        })
        .into(),
        client::Message::UpdateFiles(client::UpdateFiles {
            file_list: vec![file(), file()],
        })
        .into(),
        client::Message::Disconnect(client::Disconnect {}).into(),
        client::Message::RequestFile(client::RequestFile {
            file: PathBuf::from("file.txt"),
        })
        .into(),
        server::Message::RegisterPeer(server::RegisterPeer {
            sock: "10.134.213.134:49583".parse().unwrap(),
            file_list: vec![file()],
        })
        .into(),
        server::Message::UpdatePeer(server::UpdatePeer {
            sock: "10.134.213.134:49583".parse().unwrap(),
            file_list: vec![file()],
        })
        .into(),
        server::Message::UnregisterPeer(server::UnregisterPeer {
            sock: "10.134.213.134:49583".parse().unwrap(),
        })
        .into(),
    ];
    for msg in msgs {
        test_serialize_deserialize(msg)?;
    }
    Ok(())
}
