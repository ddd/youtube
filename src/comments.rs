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
use std::collections::HashMap;

pub struct GetCommentsRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Empty<Bytes>>,
    pub ip: &'a str,
    pub fields: GoogleAPIRequestFields<'a>,
    pub channel_id: String,
    pub page_token: Option<String>
}

impl<'a> AsMut<GoogleAPIRequestFields<'a>> for GetCommentsRequest<'a> {
    fn as_mut(&mut self) -> &mut GoogleAPIRequestFields<'a> {
        &mut self.fields
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
    items: Vec<ThreadItem>,
}

#[derive(Debug, Deserialize)]
struct ThreadItem {
    snippet: ThreadSnippet,
    replies: Option<Replies>,
}

#[derive(Debug, Deserialize)]
struct ThreadSnippet {
    #[serde(rename = "topLevelComment")]
    top_level_comment: TopLevelComment,
    #[serde(rename = "totalReplyCount")]
    total_reply_count: i32,
}

#[derive(Debug, Deserialize)]
struct TopLevelComment {
    id: String,
    snippet: CommentSnippet,
}

#[derive(Debug, Deserialize)]
struct Replies {
    comments: Vec<ReplyComment>,
}

#[derive(Debug, Deserialize)]
struct ReplyComment {
    id: String,
    snippet: CommentSnippet,
}

#[derive(Debug, Deserialize)]
struct CommentSnippet {
    #[serde(rename = "videoId")]
    video_id: Option<String>,
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
pub struct GetCommentsResponse {
    pub next_page_token: Option<String>,
    pub comments: Vec<Comment>,
}

impl<'a> GetCommentsRequest<'a> {
    pub async fn send(self) -> Result<GetCommentsResponse, YouTubeError> {
        let mut url = format!(
            "https://{}/youtube/v3/commentThreads?maxResults=100&allThreadsRelatedToChannelId={}&textFormat=plainText&order=time&part=snippet,replies",
            self.ip, self.channel_id
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
                "nextPageToken,items(snippet(topLevelComment(id,snippet(videoId,textOriginal,authorChannelId.value,likeCount,publishedAt,updatedAt)),totalReplyCount),replies.comments(id,snippet(textOriginal,authorChannelId.value,likeCount,publishedAt,updatedAt)))"
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
            let user_id = item.snippet.top_level_comment.snippet.author_channel_id.value
                .strip_prefix("UC")
                .unwrap_or(&item.snippet.top_level_comment.snippet.author_channel_id.value)
                .to_string();

            let comment_id = item.snippet.top_level_comment.id
                .strip_prefix("Ug")
                .and_then(|s| s.strip_suffix("4AaABAg"))
                .unwrap_or(&item.snippet.top_level_comment.id)
                .to_string();

            if user_id.is_empty() {
                println!("(empty user id) comment_id: {}, user_id: {}, video_id: {}", comment_id, user_id, item.snippet.top_level_comment.snippet.video_id.clone().unwrap_or_default());
                continue
            }

            // Add top-level comment
            let thread_comment = Comment {
                user_id,
                comment_id,
                parent_user_id: None,
                parent_comment_id: None,
                video_id: item.snippet.top_level_comment.snippet.video_id.clone(),
                text: item.snippet.top_level_comment.snippet.text_original,
                like_count: item.snippet.top_level_comment.snippet.like_count,
                reply_count: item.snippet.total_reply_count,
                published_at: item.snippet.top_level_comment.snippet.published_at.timestamp(),
                updated_at: if item.snippet.top_level_comment.snippet.updated_at == item.snippet.top_level_comment.snippet.published_at {
                    None
                } else {
                    Some(item.snippet.top_level_comment.snippet.updated_at.timestamp())
                },
            };
            comments.push(thread_comment);

            if let Some(replies) = item.replies {
                for reply in replies.comments {
                    let (mut parent_id, reply_id) = reply.id.split_once('.')
                        .unwrap_or((&reply.id, ""));

                    parent_id = parent_id
                        .strip_prefix("Ug")
                        .and_then(|s| s.strip_suffix("4AaABAg"))
                        .unwrap_or(&parent_id);

                    if reply.snippet.author_channel_id.value.is_empty() {
                        println!("(empty user id) reply_id: {}, user_id: {}, video_id: {}", reply_id, reply.snippet.author_channel_id.value, item.snippet.top_level_comment.snippet.video_id.clone().unwrap_or_default());
                        continue
                    }

                    let reply_comment = Comment {
                        user_id: reply.snippet.author_channel_id.value
                            .strip_prefix("UC")
                            .unwrap_or(&reply.snippet.author_channel_id.value)
                            .to_string(),
                        comment_id: reply_id.to_string(),
                        parent_user_id: Some(
                            item.snippet.top_level_comment.snippet.author_channel_id.value
                                .strip_prefix("UC")
                                .unwrap_or(&item.snippet.top_level_comment.snippet.author_channel_id.value)
                                .to_string()
                        ),
                        parent_comment_id: Some(parent_id.to_string()),
                        video_id: item.snippet.top_level_comment.snippet.video_id.clone(),
                        text: reply.snippet.text_original,
                        like_count: reply.snippet.like_count,
                        reply_count: 0,  // Replies can't have replies
                        published_at: reply.snippet.published_at.timestamp(),
                        updated_at: if reply.snippet.updated_at == reply.snippet.published_at {
                            None
                        } else {
                            Some(reply.snippet.updated_at.timestamp())
                        },
                    };
                    comments.push(reply_comment);
                }
            }
        }

        Ok(GetCommentsResponse {
            next_page_token: api_response.next_page_token,
            comments,
        })
    }
}

// New structs for getting video IDs by comment IDs
pub struct GetCommentVideoIdsRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Empty<Bytes>>,
    pub ip: &'a str,
    pub fields: GoogleAPIRequestFields<'a>,
    pub comment_ids: Vec<String>
}

impl<'a> AsMut<GoogleAPIRequestFields<'a>> for GetCommentVideoIdsRequest<'a> {
    fn as_mut(&mut self) -> &mut GoogleAPIRequestFields<'a> {
        &mut self.fields
    }
}

#[derive(Debug, Deserialize)]
struct CommentVideoIdsResponse {
    items: Vec<CommentVideoIdItem>,
}

#[derive(Debug, Deserialize)]
struct CommentVideoIdItem {
    id: String,
    snippet: CommentVideoIdSnippet,
}

#[derive(Debug, Deserialize)]
struct CommentVideoIdSnippet {
    #[serde(rename = "videoId")]
    video_id: String,
    #[serde(rename = "topLevelComment")]
    top_level_comment: TopLevelCommentInfo,
}

#[derive(Debug, Deserialize)]
struct TopLevelCommentInfo {
    snippet: TopLevelCommentSnippet,
}

#[derive(Debug, Deserialize)]
struct TopLevelCommentSnippet {
    #[serde(rename = "authorChannelId")]
    author_channel_id: AuthorChannelId,
}

#[derive(Debug)]
pub struct GetCommentVideoIdsResponse {
    pub comment_id_to_video_user: HashMap<String, String>,
}

impl<'a> GetCommentVideoIdsRequest<'a> {
    pub async fn send(self) -> Result<GetCommentVideoIdsResponse, YouTubeError> {
        // Join comment IDs with commas
        let comment_ids = self.comment_ids.join(",");
        
        let url = format!(
            "https://{}/youtube/v3/commentThreads?part=snippet,id&id={}",
            self.ip, comment_ids
        );

        let mut request_builder = Request::builder()
            .method(Method::GET)
            .uri(url)
            .header("Host", "youtube.googleapis.com")
            .header("X-Goog-Fieldmask", "items(id,snippet(videoId,topLevelComment.snippet.authorChannelId.value))");

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
        let api_response: CommentVideoIdsResponse = serde_json::from_slice(&body_bytes)?;

        let mut comment_id_to_video_user = HashMap::new();

        for item in api_response.items {
            let user_id = item.snippet.top_level_comment.snippet.author_channel_id.value
                .strip_prefix("UC")
                .unwrap_or(&item.snippet.top_level_comment.snippet.author_channel_id.value)
                .to_string();
            
            // Create the combined value of video_id:user_id
            let video_user = format!("{}:{}", item.snippet.video_id, user_id);
            
            comment_id_to_video_user.insert(item.id, video_user);
        }

        Ok(GetCommentVideoIdsResponse {
            comment_id_to_video_user,
        })
    }
}