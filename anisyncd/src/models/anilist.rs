use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AniListUserIdQuery {
    pub data: ViewerData,
}

#[derive(Deserialize, Debug)]
pub struct ViewerData {
    #[serde(rename = "Viewer")]
    pub viewer: Viewer,
}

#[derive(Deserialize, Debug)]
pub struct Viewer {
    pub id: u32,
}

#[derive(Deserialize, Debug)]
pub struct AnilistQuery {
    pub data: MediaListCollectionData,
}

#[derive(Deserialize, Debug)]
pub struct MediaListCollectionData {
    #[serde(rename = "MediaListCollection")]
    pub media_list_collection: MediaListCollection,
}

#[derive(Deserialize, Debug)]
pub struct MediaListCollection {
    pub lists: Vec<MediaiListGroup>,

    #[serde(rename = "hasNextChunk")]
    has_next_chunk: bool,
}

#[derive(Deserialize, Debug)]
pub struct MediaiListGroup {
    pub entries: Vec<MediaList>,
    pub status: String,
}

#[derive(Deserialize, Debug)]
pub struct MediaList {
    pub id: u32,
    pub score: f32,
    pub progress: u16,
    pub media: Media,
}

#[derive(Deserialize, Debug)]
pub struct Media {
    #[serde(rename = "idMal")]
    pub id_mal: u32,
    pub title: MediaTitle,
}

#[derive(Deserialize, Debug)]
pub struct MediaTitle {
    pub english: String,
}
