use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    time::SystemTime,
};

use wincode::{SchemaRead, SchemaWrite};

#[derive(Debug, SchemaWrite, SchemaRead)]
pub enum IpcCommand {
    SyncNow,
    Status,
}

#[derive(Debug, SchemaWrite, SchemaRead)]
pub enum IpcResponse {
    Ok(String),
    Err(String),
    Status(RunTimeInfo),
    Busy,
}

#[derive(Clone, Debug, SchemaWrite, SchemaRead)]
pub struct RunTimeInfo {
    pub working: bool,
    pub last_sync: Option<SystemTime>,
}

impl IpcCommand {
    pub fn recv_cmd(mut socket: &UnixStream) -> Result<Self, String> {
        let mut buffer = [0u8; 512];
        let bytes_read = socket
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read socket: {e}"))?;

        if bytes_read == 0 {
            return Err("Client disconnected".to_string());
        }

        wincode::deserialize::<IpcCommand>(&buffer[..bytes_read])
            .map_err(|e| format!("Failed to deserialize IPC command {e}"))
    }
    pub fn send_command(&self, mut socket: &UnixStream) -> Result<(), String> {
        let data = wincode::serialize(self)
            .map_err(|e| format!("Failed to serialize IPC command: {e}"))?;
        socket
            .write_all(&data)
            .map_err(|e| format!("Failed to write to socket: {e}"))?;
        Ok(())
    }
}

impl IpcResponse {
    pub fn recv_response(mut socket: &UnixStream) -> Result<Self, String> {
        let mut buffer = [0u8; 512];
        let bytes_read = socket
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read socket: {e}"))?;

        if bytes_read == 0 {
            return Err("Client disconnected".to_string());
        }

        wincode::deserialize::<IpcResponse>(&buffer[..bytes_read])
            .map_err(|e| format!("Failed to deserialize IPC response: {e}"))
    }
    pub fn send_response(&self, mut socket: &UnixStream) -> Result<(), String> {
        let data = wincode::serialize(self)
            .map_err(|e| format!("Failed to serialize IPC response: {e}"))?;
        socket
            .write_all(&data)
            .map_err(|e| format!("Failed to write to socket: {e}"))?;
        Ok(())
    }
}
