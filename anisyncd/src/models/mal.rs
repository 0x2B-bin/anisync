use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MalList {
    data: Vec<MalListEntry>,
}

#[derive(Deserialize, Debug)]
pub struct MalListEntry {
    node: AnimeNode,
    list_status: ListStatus,
}

#[derive(Deserialize, Debug)]
pub struct ListStatus {
    status: String,
    score: u8,
    num_episodes_watched: u16,
    is_rewatching: bool,
    updated_at: String,
}

#[derive(Deserialize, Debug)]
pub struct AnimeNode {
    id: u32,
    title: String,
}
