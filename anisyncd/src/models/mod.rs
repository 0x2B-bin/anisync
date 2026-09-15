use std::{collections::HashMap, str::FromStr};

use crate::models::{anilist::AnilistQuery, mal::MalList};

pub mod anilist;
pub mod mal;

#[derive(Debug)]
pub struct AnimeNode {
    pub name: String,
    pub id: u32,
    pub provider_id: u32,
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
                    provider_id: entry.node.id,
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
                        provider_id: entry.id,
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
