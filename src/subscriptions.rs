use crate::{GoogleAPIRequestFields, YouTubeError};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper::body::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::{Method, Request, StatusCode};
use chrono::{DateTime, Utc};
use serde::Deserialize;

pub struct GetSubscriptionsRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Empty<Bytes>>,
    pub ip: &'a str,
    pub fields: GoogleAPIRequestFields<'a>,
    pub channel_id: String,
    pub page_token: Option<String>
}

impl<'a> AsMut<GoogleAPIRequestFields<'a>> for GetSubscriptionsRequest<'a> {
    fn as_mut(&mut self) -> &mut GoogleAPIRequestFields<'a> {
        &mut self.fields
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
    items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Item {
    snippet: Snippet,
}

#[derive(Debug, Deserialize)]
struct Snippet {
    #[serde(rename = "publishedAt")]
    published_at: DateTime<Utc>,
    #[serde(rename = "resourceId")]
    resource_id: ResourceId,
}

#[derive(Debug, Deserialize)]
struct ResourceId {
    #[serde(rename = "channelId")]
    channel_id: String,
}

#[derive(Debug, Clone)]
pub struct Subscription {
    pub channel_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct GetSubscriptionsResponse {
    pub next_page_token: Option<String>,
    pub subscriptions: Vec<Subscription>,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: Error,
}

#[derive(Debug, Deserialize)]
struct Error {
    message: String,
}

impl<'a> GetSubscriptionsRequest<'a> {

    pub async fn send(self) -> Result<GetSubscriptionsResponse, YouTubeError> {

        let mut url = format!("https://{}/youtube/v3/subscriptions?part=snippet&order=alphabetical&channelId={}&maxResults=50", self.ip, self.channel_id);
        
        // if there is a page_token
        if let Some(page_token) = self.page_token {
            url = format!("{}&pageToken={}", url, page_token);
        }

        let mut request_builder = Request::builder()
            .method(Method::GET)
            .uri(url)
            .header("Host", "youtube.googleapis.com")
            .header("X-Goog-Fieldmask", "nextPageToken,items.snippet(publishedAt,resourceId.channelId)");

        // Only add the API key header if it's present
        if let Some(key) = self.fields.key {
            request_builder = request_builder.header("X-Goog-Api-Key", key);
        }

        if let Some(referrer) = self.fields.referrer {
            if !referrer.is_empty() {
                request_builder = request_builder.header("Referer", referrer);
            }
        }

        // Only add the Authorization header if a bearer is present
        if let Some(bearer_token) = self.fields.bearer_token {
            request_builder = request_builder.header("Authorization", bearer_token);
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
                            "Subscriptions could not be retrieved because the subscriber's account is closed." => {
                                return Err(YouTubeError::AccountClosed)
                            },
                            "Subscriptions could not be retrieved because the subscriber's account is suspended." => {
                                return Err(YouTubeError::AccountTerminated)
                            },
                            "The requester is not allowed to access the requested subscriptions." => {
                                return Err(YouTubeError::SubscriptionsPrivate)
                            },
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

        // Transform the data into our desired format
        let subscriptions: Vec<Subscription> = api_response.items
            .into_iter()
            .map(|item| Subscription {
                channel_id: item.snippet.resource_id.channel_id,
                timestamp: item.snippet.published_at.timestamp(),
            })
            .collect();

        Ok(GetSubscriptionsResponse {
            next_page_token: api_response.next_page_token,
            subscriptions,
        })
    
    }

}