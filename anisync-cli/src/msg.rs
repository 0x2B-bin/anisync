use std::os::unix::net::UnixStream;

use anisync_lib::ipc::{IpcCommand, IpcResponse};
use color_eyre::eyre::{Context, Result, eyre};

use crate::cli::IpcMsg;

pub fn run(msg: &IpcMsg) -> Result<()> {
    let socket =
        UnixStream::connect("/tmp/anisync.sock").wrap_err("Failed to connect to anisync socket")?;
    match msg {
        IpcMsg::Sync => {
            IpcCommand::SyncNow
                .send_command(&socket)
                .map_err(|e| eyre!(e))
                .wrap_err("Failed to dispatch SyncNow command over IPC".to_string())?;

            let response = IpcResponse::recv_response(&socket).map_err(|e| eyre!(e))?;

            match response {
                IpcResponse::Ok(res) => println!("{res}"),
                IpcResponse::Busy => println!("Sync rejected, daemon busy"),
                _ => println!("Unexpected response"),
            }
        }
    }

    Ok(())
}
