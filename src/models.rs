
#[derive(Debug, Clone)]
pub struct Channel {
    pub user_id: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub handle: Option<String>,
    pub profile_picture: Option<String>,
    pub banner: Option<String>,
    pub created_at: i64,
    pub country: Option<String>,
    pub view_count: i64,
    pub subscriber_count: i64,
    pub video_count: i64,
    pub topic_ids: Vec<String>,
    pub made_for_kids: bool,
    pub keywords: Option<String>,
    pub trailer: Option<String>,
    pub analytics_account_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Video {
    pub video_id: String,
    pub user_id: Option<String>,
    pub created_at: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub default_language: String,
    pub default_audio_language: String,
    pub upload_status: i8,
    pub privacy_status: i8,
    pub views: i64,
    pub likes: i64,
    pub comments: i64,
    pub made_for_kids: Option<bool>
}

#[derive(Debug, serde::Serialize)]
pub struct Comment {
    pub user_id: String,
    pub comment_id: String,
    pub parent_user_id: Option<String>,
    pub parent_comment_id: Option<String>,
    pub video_id: Option<String>,
    pub text: String,
    pub like_count: i32,
    pub reply_count: i32,
    pub updated_at: Option<i64>,
    pub published_at: i64
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PlaylistItem {
    pub kind: String,
    pub etag: String,
    pub id: String,
    pub snippet: PlaylistItemSnippet,
    #[serde(rename = "contentDetails")]
    pub content_details: PlaylistItemContentDetails,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PlaylistItemSnippet {
    #[serde(rename = "publishedAt")]
    pub published_at: String,
    #[serde(rename = "channelId")]
    pub channel_id: String,
    pub title: String,
    pub description: String,
    pub thumbnails: PlaylistItemThumbnails,
    #[serde(rename = "channelTitle")]
    pub channel_title: String,
    #[serde(rename = "playlistId")]
    pub playlist_id: String,
    pub position: u32,
    #[serde(rename = "resourceId")]
    pub resource_id: PlaylistItemResourceId,
    #[serde(rename = "videoOwnerChannelTitle")]
    pub video_owner_channel_title: Option<String>,
    #[serde(rename = "videoOwnerChannelId")]
    pub video_owner_channel_id: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PlaylistItemThumbnails {
    pub default: Option<PlaylistItemThumbnail>,
    pub medium: Option<PlaylistItemThumbnail>,
    pub high: Option<PlaylistItemThumbnail>,
    pub standard: Option<PlaylistItemThumbnail>,
    pub maxres: Option<PlaylistItemThumbnail>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PlaylistItemThumbnail {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PlaylistItemResourceId {
    pub kind: String,
    #[serde(rename = "videoId")]
    pub video_id: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PlaylistItemContentDetails {
    #[serde(rename = "videoId")]
    pub video_id: String,
    #[serde(rename = "videoPublishedAt")]
    pub video_published_at: String,
}