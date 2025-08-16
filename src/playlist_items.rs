use crate::{GoogleAPIRequestFields, YouTubeError};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper::body::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::{Method, Request, StatusCode};
use serde::Deserialize;
use crate::models::*;

pub struct ListPlaylistItemsRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Empty<Bytes>>,
    pub ip: &'a str,
    pub fields: GoogleAPIRequestFields<'a>,
    pub playlist_id: String,
    pub page_token: Option<String>,
    pub max_results: Option<u32>,
}

impl<'a> AsMut<GoogleAPIRequestFields<'a>> for ListPlaylistItemsRequest<'a> {
    fn as_mut(&mut self) -> &mut GoogleAPIRequestFields<'a> {
        &mut self.fields
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    kind: String,
    etag: String,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
    #[serde(rename = "prevPageToken")]
    prev_page_token: Option<String>,
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
    items: Vec<PlaylistItem>,
}

#[derive(Debug, Deserialize)]
struct PageInfo {
    #[serde(rename = "totalResults")]
    total_results: u32,
    #[serde(rename = "resultsPerPage")]
    results_per_page: u32,
}

#[derive(Debug)]
pub struct ListPlaylistItemsResponse {
    pub kind: String,
    pub etag: String,
    pub next_page_token: Option<String>,
    pub prev_page_token: Option<String>,
    pub total_results: u32,
    pub results_per_page: u32,
    pub items: Vec<PlaylistItem>,
}

impl<'a> ListPlaylistItemsRequest<'a> {
    pub async fn send(self) -> Result<ListPlaylistItemsResponse, YouTubeError> {
        let mut url = format!(
            "https://{}/youtube/v3/playlistItems?part=snippet%2Cid%2CcontentDetails&playlistId={}",
            self.ip, self.playlist_id
        );

        if let Some(page_token) = &self.page_token {
            url.push_str(&format!("&pageToken={}", urlencoding::encode(page_token)));
        }

        if let Some(max_results) = self.max_results {
            url.push_str(&format!("&maxResults={}", max_results));
        }

        if let Some(key) = self.fields.key {
            url.push_str(&format!("&key={}", key));
        }

        let mut req_builder = Request::builder()
            .method(Method::GET)
            .uri(url);

        if let Some(bearer_token) = self.fields.bearer_token {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", bearer_token));
        }

        if let Some(referrer) = self.fields.referrer {
            req_builder = req_builder.header("Referer", referrer);
        }

        let req = req_builder
            .body(Empty::<Bytes>::new())
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let res = self.client.request(req).await?;
        let status = res.status();

        match status {
            StatusCode::OK => {},
            StatusCode::NOT_FOUND => return Err(YouTubeError::NotFound),
            StatusCode::UNAUTHORIZED => return Err(YouTubeError::Unauthorized),
            StatusCode::FORBIDDEN => return Err(YouTubeError::Forbidden),
            StatusCode::TOO_MANY_REQUESTS => return Err(YouTubeError::Ratelimited),
            StatusCode::INTERNAL_SERVER_ERROR => return Err(YouTubeError::InternalServerError),
            _ => return Err(YouTubeError::UnknownStatusCode(status)),
        }

        let body = res.collect().await?.to_bytes();
        let body_str = std::str::from_utf8(&body)
            .map_err(|e| YouTubeError::ParseError(e.to_string()))?;

        let api_response: ApiResponse = serde_json::from_str(body_str)?;

        Ok(ListPlaylistItemsResponse {
            kind: api_response.kind,
            etag: api_response.etag,
            next_page_token: api_response.next_page_token,
            prev_page_token: api_response.prev_page_token,
            total_results: api_response.page_info.total_results,
            results_per_page: api_response.page_info.results_per_page,
            items: api_response.items,
        })
    }
}