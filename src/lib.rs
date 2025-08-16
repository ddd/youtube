use std::error::Error;
use hyper::body::Bytes;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use http_body_util::{Empty, Full};
use hyper::StatusCode;
use thiserror::Error;
use native_tls::TlsConnector;
use hyper_util::rt::TokioExecutor;

#[cfg(test)]
mod tests;

pub mod models;
pub mod utils;
pub mod videos;
pub use videos::ListVideosRequest;
pub mod channels;
pub use channels::ListChannelsRequest;
pub mod comments;
pub use comments::{GetCommentsRequest, GetCommentsResponse, GetCommentVideoIdsRequest, GetCommentVideoIdsResponse};
pub mod comment_replies;
pub use comment_replies::{GetCommentRepliesRequest, GetCommentRepliesResponse};
pub mod subscriptions;
pub use subscriptions::{GetSubscriptionsRequest, GetSubscriptionsResponse};
pub mod channel_section;
pub use channel_section::{DeleteChannelSectionRequest, CreateChannelSectionRequest};
pub mod playlist_items;
pub use playlist_items::{ListPlaylistItemsRequest, ListPlaylistItemsResponse};

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Network error: {0}")]
    NetworkError(#[from] std::io::Error),
    #[error("TLS error: {0}")]
    TlsError(#[from] native_tls::Error),
    #[error("Invalid IP: {0}")]
    InvalidIp(String)
}

pub fn initialize_client() -> Result<Client<HttpsConnector<HttpConnector>, Empty<Bytes>>, ClientError> {
    let mut http = HttpConnector::new();
    http.enforce_http(false);
    
    //if let Some(subnet) = subnet {
    //    let random_ip = crate::utils::get_rand_ipv6(subnet, subnet_id.unwrap_or_default()).map_err(|e| ClientError::InvalidIp(e.to_string()))?;
    //    http.set_local_address(Some(random_ip));
    //}
    
    let tls = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    // Create an HTTPS connector using the HTTP connector and the custom TLS connector
    let https = HttpsConnector::from((http, tls.into()));
    
    // Create the client with the custom service
    let client = Client::builder(TokioExecutor::new())
        .build::<_, Empty<Bytes>>(https);

    Ok(client)
}

pub fn initialize_full_client() -> Result<Client<HttpsConnector<HttpConnector>, Full<Bytes>>, ClientError> {
    let mut http = HttpConnector::new();
    http.enforce_http(false);
    
    //if let Some(subnet) = subnet {
    //    let random_ip = crate::utils::get_rand_ipv6(subnet, subnet_id.unwrap_or_default()).map_err(|e| ClientError::InvalidIp(e.to_string()))?;
    //    http.set_local_address(Some(random_ip));
    //}
    
    let tls = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    // Create an HTTPS connector using the HTTP connector and the custom TLS connector
    let https = HttpsConnector::from((http, tls.into()));
    
    // Create the client with the custom service
    let client = Client::builder(TokioExecutor::new())
        .build::<_, Full<Bytes>>(https);

    Ok(client)
}

pub struct GoogleAPIRequestFields<'a> {
    pub bearer_token: Option<&'a str>,
    pub key: Option<&'a str>,
    pub referrer: Option<&'a str>,
}

pub trait GoogleAPIRequest<'a> {
    fn bearer_token(&mut self) -> &mut Option<&'a str>;

    fn key(&mut self) -> &mut Option<&'a str>;

    fn referrer(&mut self) -> &mut Option<&'a str>;

    fn with_bearer_token(mut self, bearer_token: &'a str) -> Self
    where
        Self: Sized,
    {
        *self.bearer_token() = Some(bearer_token);
        self
    }

    fn with_key(mut self, key: &'a str) -> Self
    where
        Self: Sized,
    {
        *self.key() = Some(key);
        self
    }

    fn with_referrer(mut self, referrer: &'a str) -> Self
    where
        Self: Sized,
    {
        *self.referrer() = Some(referrer);
        self
    }
}

impl<'a, T> GoogleAPIRequest<'a> for T
where
    T: AsMut<GoogleAPIRequestFields<'a>>,
{
    fn bearer_token(&mut self) -> &mut Option<&'a str> {
        &mut self.as_mut().bearer_token
    }

    fn key(&mut self) -> &mut Option<&'a str> {
        &mut self.as_mut().key
    }

    fn referrer(&mut self) -> &mut Option<&'a str> {
        &mut self.as_mut().referrer
    }
}

#[derive(Error, Debug)]
pub enum YouTubeError {
    #[error("Not found")]
    NotFound,
    #[error("Ratelimited")]
    Ratelimited,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Internal server error")]
    InternalServerError,
    #[error("Unknown Status Code")]
    UnknownStatusCode(StatusCode),
    #[error("Account Closed")]
    AccountClosed,
    #[error("Account Terminated")]
    AccountTerminated,
    #[error("Subscriptions Private")] // Subscriptions specific
    SubscriptionsPrivate,
    #[error("Parse error")]
    ParseError(String),
    #[error("HTTP error: {0}")]
    HttpError(#[from] hyper::Error),
    #[error("Legacy HTTP error: {0}")]
    LegacyHttpError(#[from] hyper_util::client::legacy::Error),
    #[error("Parse error: {0}")]
    ProtobufError(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    Other(Box<dyn Error + Send + Sync>),
}

pub struct YouTubeDataV3Client {
    client: Client<HttpsConnector<HttpConnector>, Empty<Bytes>>,
    ip: String
}

impl YouTubeDataV3Client {
    pub async fn new(ip: String, client: Client<HttpsConnector<HttpConnector>, Empty<Bytes>>) -> Self {
        YouTubeDataV3Client {
            client,
            ip
        }
    }

    pub fn get_subscriptions<'a>(&'a mut self, channel_id: String, page_token: Option<String>) -> GetSubscriptionsRequest<'a> {
        GetSubscriptionsRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: GoogleAPIRequestFields{
                bearer_token: None,
                key: None,
                referrer: None
            },
            channel_id,
            page_token
        }
    }

    pub fn get_comments<'a>(&'a mut self, channel_id: String, page_token: Option<String>) -> GetCommentsRequest<'a> {
        GetCommentsRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: GoogleAPIRequestFields{
                bearer_token: None,
                key: None,
                referrer: None
            },
            channel_id,
            page_token
        }
    }

    pub fn get_comment_video_ids<'a>(&'a mut self, comment_ids: Vec<String>) -> GetCommentVideoIdsRequest<'a> {
        GetCommentVideoIdsRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: GoogleAPIRequestFields{
                bearer_token: None,
                key: None,
                referrer: None
            },
            comment_ids
        }
    }

    pub fn get_comment_replies<'a>(&'a mut self, parent_id: String, page_token: Option<String>) -> GetCommentRepliesRequest<'a> {
        GetCommentRepliesRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: GoogleAPIRequestFields{
                bearer_token: None,
                key: None,
                referrer: None
            },
            parent_id,
            page_token
        }
    }

    pub fn list_videos<'a>(&'a mut self, video_ids: Vec<String>) -> ListVideosRequest<'a> {
        ListVideosRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: GoogleAPIRequestFields{
                bearer_token: None,
                key: None,
                referrer: None
            },
            video_ids
        }
    }

    pub fn list_channels<'a>(&'a mut self, channel_ids: Vec<String>) -> ListChannelsRequest<'a> {
        ListChannelsRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: GoogleAPIRequestFields{
                bearer_token: None,
                key: None,
                referrer: None
            },
            channel_ids
        }
    }

    pub fn delete_channel_section<'a>(&'a mut self, id: String) -> DeleteChannelSectionRequest<'a> {
        DeleteChannelSectionRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: GoogleAPIRequestFields{
                bearer_token: None,
                key: None,
                referrer: None
            },
            id
        }
    }

    pub fn list_playlist_items<'a>(&'a mut self, playlist_id: String, page_token: Option<String>, max_results: Option<u32>) -> ListPlaylistItemsRequest<'a> {
        ListPlaylistItemsRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: GoogleAPIRequestFields{
                bearer_token: None,
                key: None,
                referrer: None
            },
            playlist_id,
            page_token,
            max_results
        }
    }
}