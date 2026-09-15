use crate::models::{
    AnimeNode, ExtractAnimeNodes, Status,
    anilist::{AniListUserIdQuery, AnilistQuery},
    mal::MalList,
};
use anisync_lib::{
    config::Config,
    ipc::{IpcCommand, IpcResponse, RunTimeInfo},
};
use std::{
    collections::HashMap, fs, os::unix::net::UnixListener, sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, RecvTimeoutError},
    }, thread, time::{Duration, Instant, SystemTime},
};
use thiserror::Error;
use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::EnvFilter;

const SOCKET_PATH: &str = "/tmp/anisync.sock";
const SYNC_INTERVAL: Duration = Duration::from_hours(8);

const ANILIST_GET_LIST_QUERY: &str = "
query MediaListCollection($userid: Int) {
  MediaListCollection(userId: $userid, type: ANIME) {
    lists {
      entries {
        id
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

#[derive(Debug, Error)]
enum DaemonError {
    #[error("Invalid anime node status provided")]
    InvalidStatus,

    #[error("Missing {0} access token in config. Please authenicate first.")]
    MissingToken(&'static str),

    #[error("Network / HTTP Request Failed: {0}")]
    Http(#[from] ureq::Error),

    #[error("API request to {provider} failed with HTTP {code}: {message}")]
    ApiError {
        provider: &'static str,
        code: u16,
        message: String,
    },

    #[error("Failed to parse JSON or I/O operation: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
struct NodeUpdates<'a> {
    myanimelist: Vec<&'a AnimeNode>,
    anilist: Vec<&'a AnimeNode>,
}

impl<'a> NodeUpdates<'a> {
    fn from(
        mal_nodes: &'a HashMap<u32, AnimeNode>,
        anilist_nodes: &'a HashMap<u32, AnimeNode>,
    ) -> Self {
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

        anilist_nodes
            .keys()
            .filter(|k| !mal_nodes.contains_key(k))
            .for_each(|key| {
                if let Some(node) = anilist_nodes.get(key) {
                    mal_updates.push(node);
                }
            });

        Self {
            myanimelist: mal_updates,
            anilist: anilist_updates,
        }
    }

    fn push_mal(&self, token: &str) {
        for node in &self.myanimelist {
            match push_mal_node(node, token) {
                Ok(_) => info!(
                    target: "worker",
                    "(MyAnimeList) Anime synced, {}",
                    node.name
                ),
                Err(err) => error!(
                    target: "worker",
                    error = %err,
                    "(MyAnimeList) Failed to sync anime, {}",
                    node.name
                ),
            }
            std::thread::sleep(Duration::from_millis(1000));
        }
    }
    fn push_anilist(&self, token: &str) {
        for node in &self.anilist {
            match push_anilist_node(node, token) {
                Ok(_) => info!(
                    target: "worker",
                    id = node.id,
                    "(AniList) Anime synced, {}",
                    node.name
                ),
                Err(err) => error!(
                    target: "worker",
                    error = %err,
                    "(AniList) Failed to sync anime, {}",
                    node.name
                ),
            }
        }
        std::thread::sleep(Duration::from_millis(4000));
    }
}

fn resolve_anilist_id_from_mal(malid: u32, token: &str) -> Result<u32, DaemonError> {
    const ANILIST_GET_ID_FROM_MAL: &str = "
    query GetMediaIdFromMal($malId: Int) {
      Media(idMal: $malId, type: ANIME) {
        id
      }
    }
    ";

    let payload = serde_json::json!({
        "query": ANILIST_GET_ID_FROM_MAL,
        "variables": {
            "malId": malid
        }
    });

    let mut response = ureq::post("https://graphql.anilist.co")
        .header("Authorization", format!("Bearer {token}"))
        .send_json(payload)
        .map_err(|err| match err {
            ureq::Error::StatusCode(code) => DaemonError::ApiError {
                provider: "AniList",
                code,
                message: format!("AniList responded with HTTP error {code}"),
            },
            other => DaemonError::Http(other),
        })?;

    let json: serde_json::Value = response.body_mut().read_json()?;

    json["data"]["Media"]["id"]
        .as_u64()
        .map(|id| id as u32)
        .ok_or_else(|| DaemonError::ApiError {
            provider: "AniList",
            code: 404,
            message: format!("Could not resolve MAL ID {malid} to AniList ID"),
        })
}

fn push_mal_node(node: &AnimeNode, token: &str) -> Result<(), DaemonError> {
    if node.status == Status::INVALID {
        return Err(DaemonError::InvalidStatus);
    }

    ureq::put(format!(
        "https://api.myanimelist.net/v2/anime/{}/my_list_status",
        node.id
    ))
    .header("Authorization", format!("Bearer {token}"))
    .send_form([
        (
            "status",
            match node.status {
                Status::WATCHING => "watching",
                Status::COMPLETED => "completed",
                Status::ONHOLD => "on_hold",
                Status::DROPPED => "dropped",
                Status::PLAN => "plan_to_watch",
                Status::INVALID => unreachable!(),
            },
        ),
        ("score", node.score.to_string().as_str()),
        (
            "num_watched_episodes",
            node.episodes_watched.to_string().as_str(),
        ),
    ])
    .map_err(|err| match err {
        ureq::Error::StatusCode(code) => DaemonError::ApiError {
            provider: "MyAnimeList",
            code,
            message: format!("MAL responded with HTTP error {code}"),
        },
        other => DaemonError::Http(other),
    })?;

    Ok(())
}

fn push_anilist_node(node: &AnimeNode, token: &str) -> Result<(), DaemonError> {
    const ANILIST_MUTATION : &str = "
    mutation SaveMediaListEntry($mediaId: Int, $progress: Int, $status: MediaListStatus, $score: Float) {
        SaveMediaListEntry(mediaId: $mediaId, progress: $progress, status: $status, score: $score) {
            id
            status
            progress,
            score
        }
    }
    ";

    let anilist_id = resolve_anilist_id_from_mal(node.id, token)?;

    let payload = serde_json::json!({
        "query": ANILIST_MUTATION,
        "variables": {
            "mediaId": anilist_id,
            "progress": node.episodes_watched,
            "score": node.score,
            "status": match node.status {
                Status::WATCHING => "CURRENT",
                Status::PLAN => "PLANNING",
                Status::COMPLETED => "COMPLETED",
                Status::DROPPED => "DROPPED",
                Status::ONHOLD => "PAUSED",
                Status::INVALID => unreachable!(),
            }
        }
    });

    ureq::post("https://graphql.anilist.co")
        .header("Authorization", format!("Bearer {token}"))
        .send_json(payload)
        .map_err(|err| match err {
            ureq::Error::StatusCode(code) => DaemonError::ApiError {
                provider: "AniList",
                code,
                message: format!("AniList responded with HTTP error {code}"),
            },
            other => DaemonError::Http(other),
        })?;

    Ok(())
}

fn fetch_mal_user_list(config: &Config) -> Result<MalList, DaemonError> {
    let access_token = config
        .myanimelist
        .access_token
        .as_deref()
        .ok_or(DaemonError::MissingToken("MyAnimeList"))?;

    let mut response = ureq::get(
        "https://api.myanimelist.net/v2/users/@me/animelist?fields=list_status&limit=1000&nsfw=true",
    )
    .header("Authorization", format!("Bearer {access_token}"))
    .call()?;

    let mal = response.body_mut().read_json()?;
    Ok(mal)
}

fn fetch_anilist_user_list(config: &Config) -> Result<AnilistQuery, DaemonError> {
    let access_token = config
        .anilist
        .access_token
        .as_deref()
        .ok_or(DaemonError::MissingToken("AniList"))?;

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
            error!(target: "worker", error = %err, "Failed to load configuration");
            return;
        }
    };

    let mal = match fetch_mal_user_list(&config) {
        Ok(mal) => mal,
        Err(err) => {
            error!(target: "worker", provider = "MyAnimeList", error = %err, "Failed to fetch list");
            return;
        }
    };

    let anilist = match fetch_anilist_user_list(&config) {
        Ok(anilist) => anilist,
        Err(err) => {
            error!(target: "worker", provider = "AniList", error = %err, "Failed to fetch list");
            return;
        }
    };

    let mal_nodes = mal.extract_anime_nodes();
    let anilist_nodes = anilist.extract_anime_nodes();

    let updates = NodeUpdates::from(&mal_nodes, &anilist_nodes);

    info!(target: "worker", "{} need to be synced for MyAnimeList", updates.myanimelist.len());
    info!(target: "worker", "{} need to be synced for AniList", updates.anilist.len());

    info!(target: "worker", "Syncing MyAnimeList...");
    updates.push_mal(config.myanimelist.access_token.unwrap().as_str());

    info!(target: "worker", "Syncing AniList...");
    updates.push_anilist(config.anilist.access_token.unwrap().as_str());

    info!(target: "worker", "Sync Complete");
}

fn worker_thread(rx: Receiver<IpcCommand>, runtime: Arc<Mutex<RunTimeInfo>>) {
    loop {
        match rx.recv_timeout(SYNC_INTERVAL) {
            Ok(IpcCommand::SyncNow) => {
                {
                    let mut info = runtime.lock().unwrap();
                    info.working = true;
                }

                info!(target: "worker", "Manual sync triggered");
                run_sync();

                {
                    let mut info = runtime.lock().unwrap();
                    info.working = false;
                    info.last_sync = Some(SystemTime::now())
                }
            }
            Ok(_cmd) => unreachable!(),
            Err(RecvTimeoutError::Timeout) => {
                {
                    let mut info = runtime.lock().unwrap();
                    info.working = true;
                }

                info!(target: "worker", "Scheduled Sync interval reached");
                run_sync();

                {
                    let mut info = runtime.lock().unwrap();
                    info.working = false;
                    info.last_sync = Some(SystemTime::now())
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                info!(target: "worker", "IPC channel disconnected; shutting down worker");
                break;
            }
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Starting anisync daemon...");

    let _ = fs::remove_file(SOCKET_PATH);
    let listener = UnixListener::bind(SOCKET_PATH).expect("Failed to create socket");
    let (tx, rx) = mpsc::channel::<IpcCommand>();

    let runtime = Arc::new(Mutex::new(RunTimeInfo {
        working: false,
        last_sync: None,
    }));

    let worker_runtime = runtime.clone();

    let worker_handle = thread::spawn(|| worker_thread(rx, worker_runtime));

    info!(target: "ipc", socket = SOCKET_PATH, "Listening for IPC connections");

    for stream in listener.incoming() {
        match stream {
            Ok(socket) => match IpcCommand::recv_cmd(&socket) {
                Ok(cmd) => {
                    let response = match cmd {
                        IpcCommand::SyncNow => {
                            info!(target: "ipc", "Received manual SyncNow command");
                            let working = runtime.lock().unwrap().working;
                            if working {
                                info!(target: "ipc", "Sync already in progress, rejecting sync");
                                IpcResponse::Busy
                            } else {
                                let _ = tx.send(IpcCommand::SyncNow);
                                IpcResponse::Ok("Sync accepted".to_string())
                            }
                        }
                        IpcCommand::Status => {
                            let guard = runtime.lock().unwrap();
                            let r = (*guard).clone();
                            IpcResponse::Status(r)
                        }
                    };

                    if let Err(err) = response.send_response(&socket) {
                        warn!(target: "ipc", error = %err, "Failed to send IPC response")
                    }
                }
                Err(e) => warn!(target: "ipc", error = %e, "Failed to decode IPC command"),
            },
            Err(e) => error!(target: "ipc", error = %e, "IPC connection failed"),
        }
    }

    worker_handle.join().unwrap();
    let _ = fs::remove_file(SOCKET_PATH);
}
