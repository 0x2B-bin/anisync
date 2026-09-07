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
    data: MediaListCollectionData,
}

#[derive(Deserialize, Debug)]
pub struct MediaListCollectionData {
    #[serde(rename = "MediaListCollection")]
    media_list_collection: MediaListCollection,
}

#[derive(Deserialize, Debug)]
pub struct MediaListCollection {
    lists: Vec<MediaiListGroup>,

    #[serde(rename = "hasNextChunk")]
    has_next_chunk: bool,
}

#[derive(Deserialize, Debug)]
pub struct MediaiListGroup {
    entries: Vec<MediaList>,
    status: String,
}

#[derive(Deserialize, Debug)]
pub struct MediaList {
    score: f32,
    progress: u16,
    media: Media,
}

#[derive(Deserialize, Debug)]
pub struct Media {
    #[serde(rename = "idMal")]
    id_mal: u32,
    title: MediaTitle,
}

#[derive(Deserialize, Debug)]
pub struct MediaTitle {
    english: String,
}
