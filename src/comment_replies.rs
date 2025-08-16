use crate::{GoogleAPIRequestFields, YouTubeError};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper::body::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::{Method, Request, StatusCode};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use crate::models::Comment;

pub struct GetCommentRepliesRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Empty<Bytes>>,
    pub ip: &'a str,
    pub fields: GoogleAPIRequestFields<'a>,
    pub parent_id: String,
    pub page_token: Option<String>
}

impl<'a> AsMut<GoogleAPIRequestFields<'a>> for GetCommentRepliesRequest<'a> {
    fn as_mut(&mut self) -> &mut GoogleAPIRequestFields<'a> {
        &mut self.fields
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
    items: Vec<CommentItem>,
}

#[derive(Debug, Deserialize)]
struct CommentItem {
    id: String,
    snippet: CommentSnippet,
}

#[derive(Debug, Deserialize)]
struct CommentSnippet {
    #[serde(rename = "textOriginal")]
    text_original: String,
    #[serde(rename = "authorChannelId")]
    author_channel_id: AuthorChannelId,
    #[serde(rename = "likeCount")]
    like_count: i32,
    #[serde(rename = "publishedAt")]
    published_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct AuthorChannelId {
    value: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: Error,
}

#[derive(Debug, Deserialize)]
struct Error {
    message: String,
}

#[derive(Debug)]
pub struct GetCommentRepliesResponse {
    pub next_page_token: Option<String>,
    pub comments: Vec<Comment>,
}

impl<'a> GetCommentRepliesRequest<'a> {
    pub async fn send(self) -> Result<GetCommentRepliesResponse, YouTubeError> {
        let mut url = format!(
            "https://{}/youtube/v3/comments?part=snippet,id&maxResults=100&parentId={}&textFormat=plainText",
            self.ip, self.parent_id
        );
        
        if let Some(page_token) = self.page_token {
            url = format!("{}&pageToken={}", url, page_token);
        }

        let mut request_builder = Request::builder()
            .method(Method::GET)
            .uri(url)
            .header("Host", "youtube.googleapis.com")
            .header(
                "X-Goog-Fieldmask", 
                "nextPageToken,items(id,snippet(textOriginal,authorChannelId.value,likeCount,publishedAt,updatedAt))"
            );

        if let Some(key) = self.fields.key {
            request_builder = request_builder.header("X-Goog-Api-Key", key);
        }

        if let Some(bearer_token) = self.fields.bearer_token {
            request_builder = request_builder.header("Authorization", bearer_token);
        }

        if let Some(referrer) = self.fields.referrer {
            if !referrer.is_empty() {
                request_builder = request_builder.header("Referer", referrer);
            }
        }

        let req = request_builder
            .body(Empty::new())
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let resp = self.client.request(req).await?;

        match resp.status() {
            StatusCode::TOO_MANY_REQUESTS => return Err(YouTubeError::Ratelimited),
            StatusCode::FORBIDDEN => {
                let body_bytes = resp.into_body().collect().await?.to_bytes();
                match serde_json::from_slice::<ErrorResponse>(&body_bytes) {
                    Ok(error_response) => {
                        match error_response.error.message.as_str() {
                            msg if msg.contains("parameter has disabled comments") => {
                                return Err(YouTubeError::NotFound)
                            },
                            msg if msg.starts_with("The request cannot be completed because you have exceeded your") => {
                                return Err(YouTubeError::Ratelimited)
                            },
                            msg => {
                                eprintln!("Unknown forbidden error message: {}", msg);
                                return Err(YouTubeError::Forbidden)
                            }
                        }
                    },
                    Err(_) => return Err(YouTubeError::Forbidden),
                }
            },
            StatusCode::NOT_FOUND => return Err(YouTubeError::NotFound),
            StatusCode::UNAUTHORIZED => return Err(YouTubeError::Unauthorized),
            StatusCode::INTERNAL_SERVER_ERROR | StatusCode::SERVICE_UNAVAILABLE => {
                return Err(YouTubeError::InternalServerError)
            },
            StatusCode::OK => (),
            status => {
                let body_bytes = resp.into_body().collect().await?.to_bytes();
                let body_str = String::from_utf8_lossy(&body_bytes);
                eprintln!("Unknown status code {}: {}", status.as_u16(), body_str);
                return Err(YouTubeError::UnknownStatusCode(status));
            },
        };

        let body_bytes = resp.into_body().collect().await?.to_bytes();
        let api_response: ApiResponse = serde_json::from_slice(&body_bytes)?;

        let mut comments = Vec::new();

        for item in api_response.items {
            // Skip comments that don't have the expected ID format with a period
            let idx = match item.id.rfind('.') {
                Some(idx) => idx,
                None => {
                    println!("Skipping comment with invalid ID format: {}", item.id);
                    continue;
                }
            };
            
            let parent_id = &item.id[0..idx];
            let reply_id = &item.id[idx+1..];

            // Clean up the parent ID
            let parent_id = parent_id
                .strip_prefix("Ug")
                .and_then(|s| s.strip_suffix("4AaABAg"))
                .unwrap_or(parent_id);

            if item.snippet.author_channel_id.value.is_empty() {
                println!("(empty user id) reply_id: {}, user_id: {}", reply_id, item.snippet.author_channel_id.value);
                continue
            }

            let comment = Comment {
                user_id: item.snippet.author_channel_id.value
                    .strip_prefix("UC")
                    .unwrap_or(&item.snippet.author_channel_id.value)
                    .to_string(),
                comment_id: reply_id.to_string(),
                parent_user_id: None,
                parent_comment_id: Some(parent_id.to_string()),
                video_id: None, // The comment replies endpoint doesn't provide the video ID
                text: item.snippet.text_original,
                like_count: item.snippet.like_count,
                reply_count: 0,  // Replies can't have replies
                published_at: item.snippet.published_at.timestamp(),
                updated_at: if item.snippet.updated_at == item.snippet.published_at {
                    None
                } else {
                    Some(item.snippet.updated_at.timestamp())
                },
            };
            comments.push(comment);
        }

        Ok(GetCommentRepliesResponse {
            next_page_token: api_response.next_page_token,
            comments,
        })
    }
}