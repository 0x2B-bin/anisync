use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MalList {
    pub data: Vec<MalListEntry>,
}

#[derive(Deserialize, Debug)]
pub struct MalListEntry {
    pub node: AnimeNode,
    pub list_status: ListStatus,
}

#[derive(Deserialize, Debug)]
pub struct ListStatus {
    pub status: String,
    pub score: u8,
    pub num_episodes_watched: u16,
    //pub is_rewatching: bool,
    //pub updated_at: String,
}

#[derive(Deserialize, Debug)]
pub struct AnimeNode {
    pub id: u32,
    pub title: String,
}
