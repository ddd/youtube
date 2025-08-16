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

pub struct ListChannelsRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Empty<Bytes>>,
    pub ip: &'a str,
    pub fields: GoogleAPIRequestFields<'a>,
    pub channel_ids: Vec<String>,
}

impl<'a> AsMut<GoogleAPIRequestFields<'a>> for ListChannelsRequest<'a> {
    fn as_mut(&mut self) -> &mut GoogleAPIRequestFields<'a> {
        &mut self.fields
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    items: Option<Vec<ApiChannel>>
}

#[derive(Debug, Deserialize)]
struct ApiChannel {
    id: String,
    snippet: Option<ChannelSnippet>,
    statistics: Option<ChannelStatistics>,
    #[serde(rename = "topicDetails")]
    topic_details: Option<TopicDetails>,
    status: Option<ChannelStatus>,
    #[serde(rename = "brandingSettings")]
    branding_settings: Option<BrandingSettings>
}

#[derive(Debug, Deserialize)]
struct ChannelSnippet {
    title: Option<String>,
    description: Option<String>,
    #[serde(rename = "customUrl")]
    custom_url: Option<String>,
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
    thumbnails: Option<Thumbnails>,
    country: Option<String>
}

#[derive(Debug, Deserialize)]
struct Thumbnails {
    default: Option<Thumbnail>
}

#[derive(Debug, Deserialize)]
struct Thumbnail {
    url: Option<String>
}

#[derive(Debug, Deserialize)]
struct ChannelStatistics {
    #[serde(rename = "viewCount")]
    view_count: Option<String>,
    #[serde(rename = "subscriberCount")]
    subscriber_count: Option<String>,
    #[serde(rename = "videoCount")]
    video_count: Option<String>
}

#[derive(Debug, Deserialize)]
struct TopicDetails {
    #[serde(rename = "topicIds")]
    topic_ids: Option<Vec<String>>
}

#[derive(Debug, Deserialize)]
struct ChannelStatus {
    #[serde(rename = "madeForKids")]
    made_for_kids: Option<bool>
}

#[derive(Debug, Deserialize)]
struct BrandingSettings {
    channel: Option<ChannelBranding>,
    image: Option<ChannelImage>
}

#[derive(Debug, Deserialize)]
struct ChannelBranding {
    keywords: Option<String>,
    #[serde(rename = "trackingAnalyticsAccountId")]
    tracking_analytics_account_id: Option<String>,
    #[serde(rename = "unsubscribedTrailer")]
    unsubscribed_trailer: Option<String>
}

#[derive(Debug, Deserialize)]
struct ChannelImage {
    #[serde(rename = "bannerExternalUrl")]
    banner_external_url: Option<String>
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: Error,
}

#[derive(Debug, Deserialize)]
struct Error {
    message: String,
}

impl<'a> ListChannelsRequest<'a> {

    pub async fn send(self) -> Result<Vec<Channel>, YouTubeError> {

        let url = format!("https://{}/youtube/v3/channels?part=brandingSettings,id,snippet,statistics,status,localizations,topicDetails&id={}", self.ip, self.channel_ids.join(","));

        let mut request_builder = Request::builder()
            .method(Method::GET)
            .uri(url)
            .header("Host", "youtube.googleapis.com")
            .header("X-Goog-Fieldmask", "items(id,snippet(title,description,customUrl,publishedAt,country,thumbnails.default.url),statistics(subscriberCount,viewCount,videoCount),topicDetails.topicIds,brandingSettings(channel(keywords,unsubscribedTrailer,trackingAnalyticsAccountId),image.bannerExternalUrl),status.madeForKids)");

        if let Some(key) = self.fields.key {
            request_builder = request_builder.header("X-Goog-Api-Key", key);
        }

        if let Some(bearer_token) = self.fields.bearer_token {
            request_builder = request_builder.header("Authorization", bearer_token);
        }

        if let Some(referrer) = self.fields.referrer {
            request_builder = request_builder.header("Referer", referrer);
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
                            _ => {
                                eprintln!("Unknown forbidden error message: {}", error_response.error.message);
                                return Err(YouTubeError::Forbidden)
                            }
                        }
                    },
                    Err(e) => {
                        let body_str = String::from_utf8_lossy(&body_bytes);
                        eprintln!("Failed to parse error response: {}\nResponse body: {}", e, body_str);
                        return Err(YouTubeError::Forbidden)
                    }
                }
            },
            StatusCode::NOT_FOUND => return Err(YouTubeError::NotFound),
            StatusCode::UNAUTHORIZED => {
                let body_bytes = resp.into_body().collect().await?.to_bytes();
                let body_str = String::from_utf8_lossy(&body_bytes);
                eprintln!("Unauthorized error response: {}", body_str);
                return Err(YouTubeError::Unauthorized);
            },
            StatusCode::INTERNAL_SERVER_ERROR => {
                return Err(YouTubeError::InternalServerError);
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                return Err(YouTubeError::InternalServerError);
            }
            StatusCode::OK => (), // Continue processing
            status => {
                let body_bytes = resp.into_body().collect().await?.to_bytes();
                let body_str = String::from_utf8_lossy(&body_bytes);
                eprintln!("Unknown status code {}: {}", status.as_u16(), body_str);
                return Err(YouTubeError::UnknownStatusCode(status));
            },
        };

        let body_bytes = resp.into_body().collect().await?.to_bytes();
        let api_response: ApiResponse = serde_json::from_slice(&body_bytes)?;

        // parse the response to Vec<Channel>
        let channels: Vec<Channel> = api_response.items
            .unwrap_or_default()
            .into_iter()
            .map(|channel| {
                let profile_picture = channel.snippet
                    .as_ref()
                    .and_then(|s| s.thumbnails.as_ref())
                    .and_then(|t| t.default.as_ref())
                    .and_then(|d| d.url.as_ref())
                    .and_then(|avatar_url| {
                        // Check if it's a default profile picture
                        if !avatar_url.starts_with("https://yt") || !avatar_url.contains(".ggpht.com/ytc/") {
                            // Extract the path after ggpht.com
                            avatar_url.find(".ggpht.com/").map(|index| {
                                let stripped_url = &avatar_url[(index + ".ggpht.com/".len())..];
                                // Remove everything after and including '='
                                stripped_url.split('=').next().unwrap_or(stripped_url).to_string()
                            })
                        } else {
                            None
                        }
                    });

                let banner = channel.branding_settings
                    .as_ref()
                    .and_then(|b| b.image.as_ref())
                    .and_then(|i| i.banner_external_url.as_ref())
                    .and_then(|banner_url| {
                        // Extract the path after googleusercontent.com
                        banner_url.find(".googleusercontent.com/").map(|index| {
                            let stripped_url = &banner_url[(index + ".googleusercontent.com/".len())..];
                            // Remove everything after and including '='
                            stripped_url.split('=').next().unwrap_or(stripped_url).to_string()
                        })
                    });

                let handle = channel.snippet
                    .as_ref()
                    .and_then(|s| s.custom_url.as_ref())
                    .map(|h| h.trim_start_matches('@').to_string());

                let topic_ids = channel.topic_details
                    .and_then(|t| t.topic_ids)
                    .map(|ids| ids.into_iter()
                        .map(|id| id.trim_start_matches("/m/").to_string())
                        .collect())
                    .unwrap_or_default();

                Channel {
                    user_id: channel.id,
                    display_name: channel.snippet.as_ref().and_then(|s| s.title.clone()),
                    description: channel.snippet.as_ref().and_then(|s| s.description.clone()),
                    handle,
                    profile_picture,
                    banner,
                    created_at: channel.snippet
                        .as_ref()
                        .and_then(|s| s.published_at.as_ref())
                        .and_then(|dt| DateTime::parse_from_rfc3339(dt).ok())
                        .map(|dt| dt.timestamp())
                        .unwrap_or_default(),
                    country: channel.snippet.as_ref().and_then(|s| s.country.clone()),
                    view_count: channel.statistics
                        .as_ref()
                        .and_then(|s| s.view_count.as_ref())
                        .and_then(|v| v.parse::<i64>().ok())
                        .unwrap_or_default(),
                    subscriber_count: channel.statistics
                        .as_ref()
                        .and_then(|s| s.subscriber_count.as_ref())
                        .and_then(|v| v.parse::<i64>().ok())
                        .unwrap_or_default(),
                    video_count: channel.statistics
                        .as_ref()
                        .and_then(|s| s.video_count.as_ref())
                        .and_then(|v| v.parse::<i64>().ok())
                        .unwrap_or_default(),
                    topic_ids,
                    made_for_kids: channel.status
                        .and_then(|s| s.made_for_kids)
                        .unwrap_or_default(),
                    keywords: channel.branding_settings
                        .as_ref()
                        .and_then(|b| b.channel.as_ref())
                        .and_then(|c| c.keywords.clone()),
                    trailer: channel.branding_settings
                        .as_ref()
                        .and_then(|b| b.channel.as_ref())
                        .and_then(|c| c.unsubscribed_trailer.clone()),
                    analytics_account_id: channel.branding_settings
                        .as_ref()
                        .and_then(|b| b.channel.as_ref())
                        .and_then(|c| c.tracking_analytics_account_id.clone()),
                }
            })
            .collect();

        Ok(channels)
    
    }

}