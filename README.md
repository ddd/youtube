# youtube

A Rust wrapper for the YouTube Data API v3 that provides easy access to YouTube channels, videos, comments, and subscriptions data.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
youtube = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use youtube::{initialize_client, YouTubeDataV3Client, GoogleAPIRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the HTTP client
    let client = initialize_client()?;
    
    // Create YouTube client
    let mut youtube_client = YouTubeDataV3Client::new(
        "youtube.googleapis.com".to_string(), 
        client
    ).await;
    
    // Get channel information
    let channel_ids = vec!["UCewMTclBJZPaNEfbf-qYMGA".to_string()]; // JackSucksAtLife
    let channels = youtube_client
        .list_channels(channel_ids)
        .with_key("YOUR_API_KEY")
        .send()
        .await?;
    
    println!("Found {} channels", channels.len());
    
    Ok(())
}
```

## Examples

### Getting Channel Subscriptions

```rust
use youtube::{initialize_client, YouTubeDataV3Client, GoogleAPIRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new(
        "youtube.googleapis.com".to_string(), 
        client
    ).await;
    
    let channel_id = "UCewMTclBJZPaNEfbf-qYMGA".to_string();
    let subscriptions = youtube_client
        .get_subscriptions(channel_id, None)
        .with_key("YOUR_API_KEY")
        .send()
        .await?;
    
    println!("Next page token: {:?}", subscriptions.next_page_token);
    println!("Found {} subscriptions", subscriptions.subscriptions.len());
    
    for subscription in subscriptions.subscriptions {
        println!("Subscription: {:?}", subscription);
    }
    
    Ok(())
}
```

### Getting Video Information

```rust
use youtube::{initialize_client, YouTubeDataV3Client, GoogleAPIRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new(
        "youtube.googleapis.com".to_string(), 
        client
    ).await;
    
    let video_ids = vec!["dQw4w9WgXcQ".to_string()]; // Rick Roll
    let videos = youtube_client
        .list_videos(video_ids)
        .with_key("YOUR_API_KEY")
        .send()
        .await?;
    
    for video in videos {
        println!("Title: {:?}", video.title);
        println!("Views: {}", video.views);
        println!("Likes: {}", video.likes);
    }
    
    Ok(())
}
```

### Getting Comments

```rust
use youtube::{initialize_client, YouTubeDataV3Client, GoogleAPIRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new(
        "youtube.googleapis.com".to_string(), 
        client
    ).await;
    
    let channel_id = "UCewMTclBJZPaNEfbf-qYMGA".to_string();
    let comments = youtube_client
        .get_comments(channel_id, None)
        .with_key("YOUR_API_KEY")
        .send()
        .await?;
    
    println!("Found {} comments", comments.comments.len());
    
    for comment in comments.comments {
        println!("Comment: {}", comment.text);
        println!("Likes: {}", comment.like_count);
    }
    
    Ok(())
}
```

### Getting Comment Replies

```rust
use youtube::{initialize_client, YouTubeDataV3Client, GoogleAPIRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new(
        "youtube.googleapis.com".to_string(), 
        client
    ).await;
    
    let parent_id = "PARENT_COMMENT_ID".to_string();
    let replies = youtube_client
        .get_comment_replies(parent_id, None)
        .with_key("YOUR_API_KEY")
        .send()
        .await?;
    
    for reply in replies.replies {
        println!("Reply: {}", reply.text);
    }
    
    Ok(())
}
```

## Authentication

### API Key

Most read operations can be performed using an API key:

```rust
let result = youtube_client
    .list_channels(channel_ids)
    .with_key("YOUR_API_KEY")
    .send()
    .await?;
```

### OAuth Bearer Token

For operations requiring user authentication:

```rust
let result = youtube_client
    .delete_channel_section(section_id)
    .with_bearer_token("YOUR_BEARER_TOKEN")
    .send()
    .await?;
```

### Custom Referrer

You can also set a custom referrer header:

```rust
let result = youtube_client
    .list_videos(video_ids)
    .with_key("YOUR_API_KEY")
    .with_referrer("https://your-website.com")
    .send()
    .await?;
```

## Error Handling

The library provides comprehensive error handling through the `YouTubeError` enum:

```rust
use youtube::{YouTubeError, initialize_client, YouTubeDataV3Client, GoogleAPIRequest};

#[tokio::main]
async fn main() {
    let client = initialize_client().unwrap();
    let mut youtube_client = YouTubeDataV3Client::new(
        "youtube.googleapis.com".to_string(), 
        client
    ).await;
    
    match youtube_client
        .get_subscriptions("invalid_channel_id".to_string(), None)
        .with_key("YOUR_API_KEY")
        .send()
        .await 
    {
        Ok(subscriptions) => {
            println!("Success: {} subscriptions", subscriptions.subscriptions.len());
        }
        Err(YouTubeError::NotFound) => {
            println!("Channel not found");
        }
        Err(YouTubeError::Ratelimited) => {
            println!("Rate limit exceeded");
        }
        Err(YouTubeError::Unauthorized) => {
            println!("Invalid API key or insufficient permissions");
        }
        Err(YouTubeError::SubscriptionsPrivate) => {
            println!("Channel subscriptions are private");
        }
        Err(e) => {
            println!("Other error: {}", e);
        }
    }
}
```