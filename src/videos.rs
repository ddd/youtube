use crate::{GoogleAPIRequestFields, YouTubeError};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper::body::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::{Method, Request, StatusCode};
use chrono::DateTime;
use serde::Deserialize;
use crate::models::*;

pub struct ListVideosRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Empty<Bytes>>,
    pub ip: &'a str,
    pub fields: GoogleAPIRequestFields<'a>,
    pub video_ids: Vec<String>,
}

impl<'a> AsMut<GoogleAPIRequestFields<'a>> for ListVideosRequest<'a> {
    fn as_mut(&mut self) -> &mut GoogleAPIRequestFields<'a> {
        &mut self.fields
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    items: Option<Vec<ApiVideo>>
}

#[derive(Debug, Deserialize)]
struct ApiVideo {
    id: String,
    snippet: Option<VideoSnippet>,
    status: Option<VideoStatus>,
    statistics: Option<VideoStatistics>
}

#[derive(Debug, Deserialize)]
struct VideoSnippet {
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
    #[serde(rename = "channelId")]
    channel_id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    tags: Option<Vec<String>>,
    #[serde(rename = "defaultLanguage")]
    default_language: Option<String>,
    #[serde(rename = "defaultAudioLanguage")]
    default_audio_language: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VideoStatus {
    #[serde(rename = "uploadStatus")]
    upload_status: Option<String>,
    #[serde(rename = "privacyStatus")]
    privacy_status: Option<String>,
    #[serde(rename = "madeForKids")]
    made_for_kids: Option<bool>
}

#[derive(Debug, Deserialize)]
struct VideoStatistics {
    #[serde(rename = "viewCount")]
    view_count: Option<String>,
    #[serde(rename = "likeCount")]
    like_count: Option<String>,
    #[serde(rename = "commentCount")]
    comment_count: Option<String>
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: Error,
}

#[derive(Debug, Deserialize)]
struct Error {
    message: String,
}

impl<'a> ListVideosRequest<'a> {
    pub async fn send(self) -> Result<Vec<Video>, YouTubeError> {
        // Preallocate the result vector with the known size
        let mut videos = Vec::with_capacity(self.video_ids.len());
        
        // Build query string
        let ids_str = self.video_ids.join(",");

        let url = format!(
            "https://{}/youtube/v3/videos?part=status,snippet,id&id={}", 
            self.ip, 
            ids_str
        );

        let mut request_builder = Request::builder()
            .method(Method::GET)
            .uri(url)
            .header("Host", "youtube.googleapis.com")
            .header("X-Goog-Fieldmask", "items(id,snippet(publishedAt,channelId,title,description,tags,defaultLanguage,defaultAudioLanguage),status(uploadStatus,privacyStatus,madeForKids),statistics(viewCount,likeCount,commentCount))");

        if let Some(key) = self.fields.key {
            request_builder = request_builder.header("X-Goog-Api-Key", key);
        }

        if let Some(bearer_token) = self.fields.bearer_token {
            request_builder = request_builder.header("Authorization", bearer_token);
        }

        let req = request_builder
            .body(Empty::new())
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let resp = self.client.request(req).await?;

        match resp.status() {
            StatusCode::OK => (),
            StatusCode::TOO_MANY_REQUESTS => return Err(YouTubeError::Ratelimited),
            StatusCode::FORBIDDEN => {
                let body_bytes = resp.into_body().collect().await?.to_bytes();
                if let Ok(error_response) = serde_json::from_slice::<ErrorResponse>(&body_bytes) {
                    if error_response.error.message.starts_with("The request cannot be completed because you have exceeded your") {
                        return Err(YouTubeError::Ratelimited);
                    }
                }
                return Err(YouTubeError::Forbidden);
            },
            StatusCode::NOT_FOUND => return Err(YouTubeError::NotFound),
            StatusCode::UNAUTHORIZED => return Err(YouTubeError::Unauthorized),
            StatusCode::INTERNAL_SERVER_ERROR | StatusCode::SERVICE_UNAVAILABLE => {
                return Err(YouTubeError::InternalServerError)
            },
            status => {
                let body_bytes = resp.into_body().collect().await?.to_bytes();
                let body_str = String::from_utf8_lossy(&body_bytes);
                eprintln!("Unknown status code {}: {}", status.as_u16(), body_str);
                return Err(YouTubeError::UnknownStatusCode(status));
            },
        };

        let body_bytes = resp.into_body().collect().await?.to_bytes();
        let api_response: ApiResponse = serde_json::from_slice(&body_bytes)?;

        // Process each requested video ID
        for video_id in self.video_ids {
            let api_video = api_response.items
                .as_ref()
                .and_then(|items| items.iter().find(|v| v.id == video_id));

            let video = match api_video {
                None => Video {
                    video_id,
                    user_id: None,
                    created_at: 0,
                    title: None,
                    description: None,
                    tags: Vec::new(),
                    default_language: String::new(),
                    default_audio_language: String::new(),
                    upload_status: 0,
                    privacy_status: 0,
                    views: 0,
                    likes: 0,
                    comments: 0,
                    made_for_kids: None,
                },
                Some(api_video) => {
                    let privacy_status = match api_video.status.as_ref().and_then(|s| s.privacy_status.as_ref()).map(|s| s.as_str()) {
                        Some("unlisted") => 1,
                        Some("public") => 2,
                        _ => 0,
                    };

                    let upload_status = match api_video.status.as_ref().and_then(|s| s.upload_status.as_ref()).map(|s| s.as_str()) {
                        Some("uploaded") => 1,
                        Some("processed") => 2,
                        _ => 0,
                    };

                    let user_id = api_video.snippet
                        .as_ref()
                        .and_then(|s| s.channel_id.as_ref())
                        .map(|id| id.strip_prefix("UC").unwrap_or(id).to_string());

                    let created_at = api_video.snippet
                        .as_ref()
                        .and_then(|s| s.published_at.as_ref())
                        .and_then(|dt| DateTime::parse_from_rfc3339(dt).ok())
                        .map(|dt| dt.timestamp())
                        .unwrap_or(0);

                    // Parse statistics with safe conversion
                    let parse_count = |s: Option<&String>| -> i64 {
                        s.and_then(|s| s.parse::<i64>().ok()).unwrap_or(0)
                    };

                    let statistics = api_video.statistics.as_ref();
                    let views = parse_count(statistics.and_then(|s| s.view_count.as_ref()));
                    let likes = parse_count(statistics.and_then(|s| s.like_count.as_ref()));
                    let comments = parse_count(statistics.and_then(|s| s.comment_count.as_ref()));

                    Video {
                        video_id,
                        user_id,
                        created_at,
                        title: api_video.snippet.as_ref().and_then(|s| s.title.clone()),
                        description: api_video.snippet.as_ref().and_then(|s| s.description.clone()),
                        tags: api_video.snippet.as_ref().and_then(|s| s.tags.clone()).unwrap_or_default(),
                        default_language: api_video.snippet.as_ref().and_then(|s| s.default_language.clone()).unwrap_or_default(),
                        default_audio_language: api_video.snippet.as_ref().and_then(|s| s.default_audio_language.clone()).unwrap_or_default(),
                        upload_status,
                        privacy_status,
                        views,
                        likes,
                        comments,
                        made_for_kids: api_video.status.as_ref().and_then(|s| s.made_for_kids),
                    }
                }
            };
            videos.push(video);
        }

        Ok(videos)
    }
}