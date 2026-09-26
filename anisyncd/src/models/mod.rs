use std::{collections::HashMap, str::FromStr};

use crate::models::{anilist::AnilistQuery, mal::MalList};

pub mod anilist;
pub mod mal;

#[derive(Debug, PartialEq)]
pub struct AnimeNode {
    pub name: String,
    pub id: u32,
    pub episodes_watched: u16,
    pub score: u8,
    pub status: Status,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    COMPLETED,
    WATCHING,
    DROPPED,
    ONHOLD,
    PLAN,
    INVALID,
}

pub trait ExtractAnimeNodes {
    fn extract_anime_nodes(&self) -> HashMap<u32, AnimeNode>;
}

impl FromStr for Status {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "completed" => Ok(Status::COMPLETED),
            "current" => Ok(Status::WATCHING),
            "watching" => Ok(Status::WATCHING),
            "dropped" => Ok(Status::DROPPED),
            "plan_to_watch" => Ok(Status::PLAN),
            "planning" => Ok(Status::PLAN),
            "repeating" => Ok(Status::COMPLETED),
            "on_hold" => Ok(Status::ONHOLD),
            "paused" => Ok(Status::ONHOLD),
            _ => Ok(Status::INVALID),
        }
    }
}

impl ExtractAnimeNodes for MalList {
    fn extract_anime_nodes(&self) -> HashMap<u32, AnimeNode> {
        let mut nodes = HashMap::new();

        for entry in &self.data {
            nodes.insert(
                entry.node.id,
                AnimeNode {
                    name: entry.node.title.clone(),
                    id: entry.node.id,
                    episodes_watched: entry.list_status.num_episodes_watched,
                    status: entry.list_status.status.parse().unwrap(),
                    score: entry.list_status.score,
                },
            );
        }

        nodes
    }
}

impl ExtractAnimeNodes for AnilistQuery {
    fn extract_anime_nodes(&self) -> HashMap<u32, AnimeNode> {
        let mut nodes = HashMap::new();

        for media_group in &self.data.media_list_collection.lists {
            let status: Status = media_group.status.parse().unwrap();

            for entry in &media_group.entries {
                nodes.insert(
                    entry.media.id_mal,
                    AnimeNode {
                        name: match &entry.media.title.english {
                            Some(english) => english.clone(),
                            None => match &entry.media.title.romaji {
                                Some(romaji) => romaji.clone(),
                                None => entry.id.to_string(),
                            },
                        },
                        id: entry.media.id_mal,
                        episodes_watched: entry.progress,
                        status: status.clone(),
                        score: entry.score as u8,
                    },
                );
            }
        }

        nodes
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{
        anilist::{
            Media, MediaList, MediaListCollection, MediaListCollectionData, MediaTitle,
            MediaiListGroup,
        },
        mal::{ListStatus, MalAnimeNode, MalListEntry},
    };

    use super::*;

    #[test]
    fn normalize_mal_list() {
        let mal_list = MalList {
            data: Vec::from([
                MalListEntry {
                    node: MalAnimeNode {
                        id: 38101,
                        title: "5-toubun no Hanayome".to_string(),
                    },
                    list_status: ListStatus {
                        status: "completed".to_string(),
                        score: 7,
                        num_episodes_watched: 12,
                    },
                },
                MalListEntry {
                    node: MalAnimeNode {
                        id: 39783,
                        title: "5-toubun no Hanayome ∬".to_string(),
                    },
                    list_status: ListStatus {
                        status: "completed".to_string(),
                        score: 8,
                        num_episodes_watched: 12,
                    },
                },
                MalListEntry {
                    node: MalAnimeNode {
                        id: 48548,
                        title: "5-toubun no Hanayome Movie".to_string(),
                    },
                    list_status: ListStatus {
                        status: "completed".to_string(),
                        score: 8,
                        num_episodes_watched: 1,
                    },
                },
            ]),
        };

        let expected_mal_nodes: HashMap<u32, AnimeNode> = HashMap::from([
            (
                38101,
                AnimeNode {
                    name: "5-toubun no Hanayome".to_string(),
                    id: 38101,
                    episodes_watched: 12,
                    score: 7,
                    status: Status::COMPLETED,
                },
            ),
            (
                39783,
                AnimeNode {
                    name: "5-toubun no Hanayome ∬".to_string(),
                    id: 39783,
                    episodes_watched: 12,
                    score: 8,
                    status: Status::COMPLETED,
                },
            ),
            (
                48548,
                AnimeNode {
                    name: "5-toubun no Hanayome Movie".to_string(),
                    id: 48548,
                    episodes_watched: 1,
                    score: 8,
                    status: Status::COMPLETED,
                },
            ),
        ]);

        let mal_nodes = mal_list.extract_anime_nodes();

        assert_eq!(mal_nodes, expected_mal_nodes);
    }

    #[test]
    fn normalize_anilist_list() {
        let anilist = AnilistQuery {
            data: MediaListCollectionData {
                media_list_collection: MediaListCollection {
                    lists: vec![MediaiListGroup {
                        entries: vec![
                            MediaList {
                                id: 437951066,
                                score: 7.0,
                                progress: 12,
                                media: Media {
                                    id_mal: 57334,
                                    title: MediaTitle {
                                        english: Some("DAN DA DAN".to_string()),
                                        romaji: None,
                                    },
                                },
                            },
                            MediaList {
                                id: 438011728,
                                score: 7.0,
                                progress: 12,
                                media: Media {
                                    id_mal: 52092,
                                    title: MediaTitle {
                                        english: Some("My Home Hero".to_string()),
                                        romaji: None,
                                    },
                                },
                            },
                            MediaList {
                                id: 438061623,
                                score: 8.0,
                                progress: 1,
                                media: Media {
                                    id_mal: 50594,
                                    title: MediaTitle {
                                        english: Some("Suzume".to_string()),
                                        romaji: None,
                                    },
                                },
                            },
                        ],
                        status: "COMPLETED".to_string(),
                    }],
                },
            },
        };

        let anilist_nodes = anilist.extract_anime_nodes();
        let expected_anilist_nodes: HashMap<u32, AnimeNode> = HashMap::from([
            (
                57334,
                AnimeNode {
                    name: "DAN DA DAN".to_string(),
                    id: 57334,
                    episodes_watched: 12,
                    score: 7,
                    status: Status::COMPLETED,
                },
            ),
            (
                52092,
                AnimeNode {
                    name: "My Home Hero".to_string(),
                    id: 52092,
                    episodes_watched: 12,
                    score: 7,
                    status: Status::COMPLETED,
                },
            ),
            (
                50594,
                AnimeNode {
                    name: "Suzume".to_string(),
                    id: 50594,
                    episodes_watched: 1,
                    score: 8,
                    status: Status::COMPLETED,
                },
            ),
        ]);

        assert_eq!(anilist_nodes, expected_anilist_nodes);
    }
}
