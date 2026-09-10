use crate::models::{
    ExtractAnimeNodes, anilist::{AniListUserIdQuery, AnilistQuery}, mal::MalList, AnimeNode
};
use anisync_lib::{config::Config, ipc::IpcCommand};
use std::{
    collections::HashMap, fs, os::unix::net::UnixListener, sync::mpsc::{self, Receiver, RecvTimeoutError}, thread, time::Duration,
};

const SOCKET_PATH: &str = "/tmp/anisync.sock";
const SYNC_INTERVAL: Duration = Duration::from_hours(8);

const ANILIST_GET_LIST_QUERY: &str = "
query MediaListCollection($userid: Int) {
  MediaListCollection(userId: $userid, type: ANIME) {
    lists {
      entries {
        progress
        score
        media {
          title {
            english
          }
          idMal
        }
      }
      status
    }
    hasNextChunk
  }
}
";

const ANILIST_GET_USER_ID_QUERY: &str = "
query GetUserId {
  Viewer {
    id
  }
}
";

mod models;

#[derive(Debug)]
struct NodeUpdates<'a> {
    myanimelist: Vec<&'a AnimeNode>,
    anilist: Vec<&'a AnimeNode>,
}

impl<'a> NodeUpdates<'a> {
    fn from(mal_nodes: &'a HashMap<u32, AnimeNode>, anilist_nodes: &'a HashMap<u32, AnimeNode>) -> Self {
        let mut mal_updates = Vec::new();
        let mut anilist_updates = Vec::new();
        for (id, mal_node) in mal_nodes {
            if let Some(anilist_node) = anilist_nodes.get(id) {
                if mal_node.status < anilist_node.status {
                    anilist_updates.push(mal_node);
                } else if anilist_node.status < mal_node.status {
                    mal_updates.push(anilist_node);
                }
            } else {
                anilist_updates.push(mal_node);
            }
        }

        anilist_nodes.keys().filter(|k| !mal_nodes.contains_key(k)).for_each(|key| {
            if let Some(node) = anilist_nodes.get(key) {
                mal_updates.push(node);
            }
        });

        Self {
            myanimelist: mal_updates,
            anilist: anilist_updates
        }
    }
}

fn fetch_mal_user_list(config: &Config) -> Result<MalList, Box<dyn std::error::Error>> {
    let access_token = config
        .myanimelist
        .access_token
        .as_deref()
        .ok_or("MyAnimeList access token missing. Please login first")?;

    let mut response = ureq::get(
        "https://api.myanimelist.net/v2/users/@me/animelist?fields=list_status&limit=1000&nsfw=true",
    )
    .header("Authorization", format!("Bearer {access_token}"))
    .call()?;

    let mal = response.body_mut().read_json()?;
    Ok(mal)
}

fn fetch_anilist_user_list(config: &Config) -> Result<AnilistQuery, Box<dyn std::error::Error>> {
    let access_token = config
        .anilist
        .access_token
        .as_deref()
        .ok_or("AniList access token missing. Please login first")?;

    let userid_query = serde_json::json!({
        "query": ANILIST_GET_USER_ID_QUERY
    });

    let mut response = ureq::post("https://graphql.anilist.co")
        .header("Authorization", format!("Bearer {access_token}"))
        .send_json(userid_query)?;

    let userid_query: AniListUserIdQuery = response.body_mut().read_json()?;

    let list_query = serde_json::json!({
        "query": ANILIST_GET_LIST_QUERY,
        "variables": { "userid": userid_query.data.viewer.id },
    });

    let mut response = ureq::post("https://graphql.anilist.co")
        .header("Authorization", format!("Bearer {access_token}"))
        .send_json(list_query)?;

    let anilist = response.body_mut().read_json()?;
    Ok(anilist)
}

fn run_sync() {
    let config = match Config::load() {
        Ok(config) => config,
        Err(err) => {
            println!("[Worker]: {err}");
            return;
        }
    };

    let mal = match fetch_mal_user_list(&config) {
        Ok(mal) => mal,
        Err(err) => {
            println!("[Worker]: Failed to fetch MyAnimeList list: {err}");
            return;
        }
    };

    let anilist = match fetch_anilist_user_list(&config) {
        Ok(anilist) => anilist,
        Err(err) => {
            println!("[Worker] {err}");
            return;
        }
    };

    let mal_nodes = mal.extract_anime_nodes();
    let anilist_nodes = anilist.extract_anime_nodes();

    let _updates = NodeUpdates::from(&mal_nodes, &anilist_nodes);
}

fn worker_thread(rx: Receiver<IpcCommand>) {
    loop {
        match rx.recv_timeout(SYNC_INTERVAL) {
            Ok(IpcCommand::SyncNow) => {
                println!("[Worker] Manual Sync Triggered through IPC");
                run_sync();
            }
            Err(RecvTimeoutError::Timeout) => {
                println!("[Worker] Scheduled Sync");
                run_sync();
            }
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
