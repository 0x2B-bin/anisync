use anisync_lib::{config::Config, ipc::IpcCommand};
use std::{
    fs,
    os::unix::net::UnixListener,
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread,
    time::Duration,
};
use serde::Deserialize;

const SOCKET_PATH: &str = "/tmp/anisync.sock";
const SYNC_INTERVAL: Duration = Duration::from_hours(8);

#[derive(Deserialize, Debug)]
struct MalList {
    data: Vec<MalListEntry>
}

#[derive(Deserialize, Debug)]
struct MalListEntry {
    node: AnimeNode,
    list_status: ListStatus
}

#[derive(Deserialize, Debug)]
struct ListStatus {
    status: String,
    score: u8,
    num_episodes_watched: u16,
    is_rewatching: bool,
    updated_at: String
}

#[derive(Deserialize, Debug)]
struct AnimeNode {
    id: u32,
    title: String,

}

fn fetch_mal_user_list(config: &Config) -> Result<MalList, Box<dyn std::error::Error>> {
    let access_token = config
        .myanimelist
        .access_token
        .as_deref()
        .ok_or("MyAnimeList access token missin. Please login first")?;
    
    let mut response = ureq::get("https://api.myanimelist.net/v2/users/@me/animelist?fields=list_status&limit=1000")
        .header("Authorization", format!("Bearer {access_token}"))
        .call()?;

    let mal = response.body_mut().read_json()?;
    Ok(mal)
}

fn run_sync() {
    let config = match Config::load() {
        Ok(config) => config,
        Err(err) => {
            println!("[Worker]: {err}");
            return
        }
    };

    let mal = match fetch_mal_user_list(&config) {
        Ok(mal) => mal,
        Err(err) => {
            println!("[Worker] {err}");
            return
        }
    };

    println!("{mal:?}");
}

fn worker_thread(rx: Receiver<IpcCommand>) {
    loop {
        match rx.recv_timeout(SYNC_INTERVAL) {
            Ok(IpcCommand::SyncNow) => {
                println!("[Worker] Manual Sync Triggered through IPC");
                run_sync();
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {}
        }
    }
}

fn main() {
    let _ = fs::remove_file(SOCKET_PATH);
    let listener = UnixListener::bind(SOCKET_PATH).expect("Failed to create socket");
    let (tx, rx) = mpsc::channel::<IpcCommand>();

    let worker_handle = thread::spawn(|| worker_thread(rx));

    println!("[IPC] Listening for connections");
    for stream in listener.incoming() {
        match stream {
            Ok(socket) => match IpcCommand::recv_cmd(&socket) {
                Ok(cmd) => match cmd {
                    IpcCommand::SyncNow => {
                        let _ = tx.send(IpcCommand::SyncNow);
                    }
                },
                Err(e) => println!("[IPC] Err: {e}"),
            },
            Err(e) => println!("[IPC] Connection Failed: {e}"),
        }
    }

    worker_handle.join().unwrap();
    let _ = fs::remove_file(SOCKET_PATH);
}
