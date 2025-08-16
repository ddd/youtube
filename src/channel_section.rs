use crate::{GoogleAPIRequestFields, YouTubeError};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper::body::Bytes;
use http_body_util::{BodyExt, Full, Empty};
use hyper::{Method, Request, StatusCode};
use serde::{Deserialize, Serialize};

pub struct DeleteChannelSectionRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Empty<Bytes>>,
    pub ip: &'a str,
    pub fields: GoogleAPIRequestFields<'a>,
    pub id: String,
}

pub struct CreateChannelSectionRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub ip: &'a str,
    pub fields: GoogleAPIRequestFields<'a>,
    pub author_channel_id: String,
    pub position: u32,
    pub channels: Vec<String>,
}

impl<'a> AsMut<GoogleAPIRequestFields<'a>> for DeleteChannelSectionRequest<'a> {
    fn as_mut(&mut self) -> &mut GoogleAPIRequestFields<'a> {
        &mut self.fields
    }
}

impl<'a> AsMut<GoogleAPIRequestFields<'a>> for CreateChannelSectionRequest<'a> {
    fn as_mut(&mut self) -> &mut GoogleAPIRequestFields<'a> {
        &mut self.fields
    }
}

#[derive(Debug, Serialize)]
struct CreateChannelSectionRequestBody {
    snippet: ChannelSectionSnippet,
    #[serde(rename = "contentDetails")]
    content_details: ChannelSectionContentDetails,
}

#[derive(Debug, Serialize)]
struct ChannelSectionSnippet {
    #[serde(rename = "type")]
    section_type: String,
    position: u32,
}

#[derive(Debug, Serialize)]
struct ChannelSectionContentDetails {
    channels: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CreateChannelSectionResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: Error,
}

#[derive(Debug, Deserialize)]
struct Error {
    message: String,
}

impl<'a> DeleteChannelSectionRequest<'a> {
    pub async fn send(self) -> Result<bool, YouTubeError> {
        let url = format!("https://{}/youtube/v3/channelSections?id={}", self.ip, self.id);

        let mut request_builder = Request::builder()
            .method(Method::DELETE)
            .uri(url)
            .header("Host", "youtube.googleapis.com");

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
            StatusCode::NO_CONTENT => return Ok(true), // Success case for DELETE
            status => {
                let body_bytes = resp.into_body().collect().await?.to_bytes();
                let body_str = String::from_utf8_lossy(&body_bytes);
                eprintln!("Unknown status code {}: {}", status.as_u16(), body_str);
                return Err(YouTubeError::UnknownStatusCode(status));
            },
        };
    }
}

impl<'a> CreateChannelSectionRequest<'a> {
    pub async fn send(self) -> Result<String, YouTubeError> {
        let url = format!("https://{}/youtube/v3/channelSections?part=snippet,id,contentDetails", self.ip);

        let request_body = CreateChannelSectionRequestBody {
            snippet: ChannelSectionSnippet {
                section_type: "multipleChannels".to_string(),
                position: self.position,
            },
            content_details: ChannelSectionContentDetails {
                channels: self.channels,
            },
        };

        let body_json = serde_json::to_string(&request_body)
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let mut request_builder = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header("Host", "youtube.googleapis.com")
            .header("Content-Type", "application/json");

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
            .body(body_json.into())
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
        let api_response: CreateChannelSectionResponse = serde_json::from_slice(&body_bytes)?;

        Ok(api_response.id)
    }
}