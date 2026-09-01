use anisync_lib::ipc::IpcCommand;
use std::{os::unix::net::UnixListener, sync::mpsc};

const SOCKET_PATH: &str = "/tmp/anisync.sock";

fn main() {
    let listener = UnixListener::bind(SOCKET_PATH).expect("Failed to create socket");
    //let (tx, rx) = mpsc::channel::<IpcCommand>();

    println!("[IPC] Listening for connections");
    for stream in listener.incoming() {
        match stream {
            Ok(socket) => match IpcCommand::recv_cmd(&socket) {
                Ok(cmd) => match cmd {
                    IpcCommand::SyncNow => println!("Got sync now"),
                },
                Err(e) => println!("[IPC] Err: {e}"),
            },
            Err(e) => println!("[IPC] Connection Failed: {e}"),
        }
    }
}
