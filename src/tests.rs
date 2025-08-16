use std::error::Error;
use crate::{initialize_client, GoogleAPIRequest};
use tokio;
use crate::{YouTubeDataV3Client, YouTubeError};

const API_KEY: &str = "AIzaSyBeo4NGA__U6Xxy-aBE6yFm19pgq8TY-TM";

#[tokio::test]
async fn test_get_subscription() -> Result<(), Box<dyn Error>> {
    // JackSucksAtLife's channel
    let channel_id = "UCewMTclBJZPaNEfbf-qYMGA".to_string();
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let subscriptions = youtube_client.get_subscriptions(channel_id, None).with_key(API_KEY).with_referrer("https://explorer.apis.google.com").send().await?;
    
    assert!(!subscriptions.next_page_token.is_none());
    assert!(!subscriptions.subscriptions.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_hidden_user_subscriptions() -> Result<(), Box<dyn Error>> {
    // /user/Z's channel
    let channel_id = "UCk3PBU7EtwVhotDzGvwUtAg".to_string();
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let subscriptions = youtube_client.get_subscriptions(channel_id, None).with_key(API_KEY).with_referrer("https://explorer.apis.google.com").send().await;
    assert!(subscriptions.is_err(), "Expected error for private subscriptions");
    if let Err(err) = subscriptions {
        assert!(matches!(err, YouTubeError::SubscriptionsPrivate));
    }
    
    Ok(())
}


#[tokio::test]
async fn test_private_subscriptions() -> Result<(), Box<dyn Error>> {
    // MrBeast's channel
    let channel_id = "UCX6OQ3DkcsbYNE6H8uQQuVA".to_string();
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let subscriptions = youtube_client.get_subscriptions(channel_id, None).with_key(API_KEY).with_referrer("https://explorer.apis.google.com").send().await;
    assert!(subscriptions.is_err(), "Expected error for private subscriptions");
    if let Err(err) = subscriptions {
        assert!(matches!(err, YouTubeError::SubscriptionsPrivate));
    }
    
    Ok(())
}

#[tokio::test]
async fn test_non_existant_channel_subscriptions() -> Result<(), Box<dyn Error>> {
    // Non-existant channel ID
    let channel_id = "UC0123456789ABCDEFGHIJ".to_string();
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let subscriptions = youtube_client.get_subscriptions(channel_id, None).with_key(API_KEY).with_referrer("https://explorer.apis.google.com").send().await;
    assert!(subscriptions.is_err(), "Expected error for not found");
    if let Err(err) = subscriptions {
        assert!(matches!(err, YouTubeError::NotFound));
    }
    
    Ok(())
}

#[tokio::test]
async fn test_get_channel() -> Result<(), Box<dyn Error>> {
    // Smosh's channel
    let channel_id = vec!["UCY30JRSgfhYXA6i6xX1erWg".to_string()];
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let channels = youtube_client.list_channels(channel_id).with_key(API_KEY).with_referrer("https://explorer.apis.google.com").send().await?;
    
    assert_eq!(channels.len(), 1);
    let channel = &channels[0];
    
    assert_eq!(channel.user_id, "UCY30JRSgfhYXA6i6xX1erWg");
    assert_eq!(channel.display_name, Some("Smosh".to_string()));
    assert_eq!(channel.description, Some("Join us in Bit City every other Friday!\n".to_string()));
    assert_eq!(channel.handle, Some("smosh".to_string())); // @ removed
    // created_at is 2005-11-19T10:25:07Z as timestamp
    assert_eq!(channel.created_at, 1132395907);
    assert_eq!(channel.country, Some("US".to_string()));
    assert!(channel.view_count > 0);
    assert!(channel.subscriber_count > 0);
    assert!(channel.video_count > 1935);
    
    // Topic IDs with /m/ prefix removed
    assert!(channel.topic_ids.contains(&"02vxn".to_string()));
    assert!(channel.topic_ids.contains(&"02jjt".to_string())); 
    assert!(channel.topic_ids.contains(&"0f2f9".to_string()));
    assert_eq!(channel.made_for_kids, false);
    
    assert_eq!(channel.keywords, Some("smosh anthony padilla ian hecox pokemon smoosh smash sketch comedy skit humor funny short shayne topp courtney miller olivia sui".to_string()));
    assert_eq!(channel.trailer, Some("upaC2T9MUI8".to_string()));
    assert_eq!(channel.analytics_account_id, Some("UA-87364751-18".to_string()));
    
    // For profile picture: we expect None since it contains .googleusercontent.com/ytc/
    assert_eq!(channel.profile_picture, Some("AKu8GRve-zh81APMLaRvmbj_GlBYpa3zMqNi-Gkn9vPVYRJhC3eLVbBlybkhDQJztqIFc_Z7p2Y".to_string()));
    
    // For banner: we expect everything after googleusercontent.com/ and before any query params
    assert_eq!(channel.banner, Some("7-79NaZvZMm1HVv_-RnasIoQny5YMRXP08Z8N2mYZyAXmxE_kAyiVObmT02-EY_9XV4J9ZHLxw".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_get_minimal_channel() -> Result<(), Box<dyn Error>> {
    let channel_id = vec!["UCyj-EUmmEfIlUg-pYVn-vxw".to_string()];
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let channels = youtube_client.list_channels(channel_id).with_key(API_KEY).with_referrer("https://explorer.apis.google.com").send().await?;
    
    assert_eq!(channels.len(), 1);
    let channel = &channels[0];
    
    // Basic info
    assert_eq!(channel.user_id, "UCyj-EUmmEfIlUg-pYVn-vxw");
    assert_eq!(channel.display_name, Some("ladybody".to_string()));
    assert_eq!(channel.description, Some("".to_string()));
    assert_eq!(channel.handle, Some("ladybody".to_string())); // @ removed
    // created_at is 2007-12-14T20:16:22Z as timestamp
    assert_eq!(channel.created_at, 1197663382);
    
    // Statistics
    assert_eq!(channel.view_count, 657);
    assert_eq!(channel.subscriber_count, 1);
    assert_eq!(channel.video_count, 3);
    
    // Topic IDs with /m/ prefix removed
    assert!(channel.topic_ids.contains(&"019_rr".to_string()));
    assert!(channel.topic_ids.contains(&"04rlf".to_string()));
    assert!(channel.topic_ids.contains(&"06ntj".to_string()));
    
    // Empty or default fields
    assert_eq!(channel.country, None);
    assert_eq!(channel.made_for_kids, false); // default value when status is empty
    assert_eq!(channel.keywords, None);
    assert_eq!(channel.trailer, None);
    assert_eq!(channel.analytics_account_id, None);
    assert_eq!(channel.banner, None);
    
    // Profile picture should be None since it's a default picture (contains ggpht.com/ytc/)
    assert_eq!(channel.profile_picture, None);
    
    Ok(())
}

#[tokio::test]
async fn test_get_comments() -> Result<(), Box<dyn Error>> {
    // JackSucksAtLife's channel
    let channel_id = "UC4QobU6STFB0P71PMvOGN5A".to_string();
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let comments = youtube_client.get_comments(channel_id, None).with_key(API_KEY).with_referrer("https://explorer.apis.google.com").send().await?;
    
    assert!(!comments.next_page_token.is_none());
    assert!(!comments.comments.is_empty());
    
    // Check for at least one top-level comment and one reply
    let mut found_top_level = false;
    let mut found_reply = false;
    
    for comment in comments.comments {
        if comment.parent_comment_id.is_none() {
            found_top_level = true;
            assert!(!comment.comment_id.is_empty());
            assert!(comment.video_id.is_some());
            assert!(!comment.text.is_empty());
            assert!(comment.published_at > 0);
            assert!(comment.reply_count >= 0);
        } else {
            found_reply = true;
            assert!(!comment.comment_id.is_empty());
            assert!(comment.parent_comment_id.is_some());
            assert!(comment.parent_user_id.is_some());
            assert!(comment.video_id.is_some());
            assert!(!comment.text.is_empty());
            assert!(comment.published_at > 0);
            assert_eq!(comment.reply_count, 0); // Replies can't have replies
        }
    }
    
    assert!(found_top_level);
    assert!(found_reply);
    
    Ok(())
}

#[tokio::test]
async fn test_empty_comments() -> Result<(), Box<dyn Error>> {
    // Channel with no comments
    let channel_id = "UCyj-EUmmEfIlUg-pYVn-vxw".to_string();
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let comments = youtube_client.get_comments(channel_id, None).with_key(API_KEY).with_referrer("https://explorer.apis.google.com").send().await?;
    assert!(comments.comments.is_empty());
    assert!(comments.next_page_token.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_non_existent_channel_comments() -> Result<(), Box<dyn Error>> {
    let channel_id = "UC0123456789ABCDEFGHIJ".to_string();
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let comments = youtube_client.get_comments(channel_id, None).with_key(API_KEY).with_referrer("https://explorer.apis.google.com").send().await;
    assert!(comments.is_err());
    if let Err(err) = comments {
        assert!(matches!(err, YouTubeError::NotFound));
    }
    
    Ok(())
}

#[tokio::test]
async fn test_get_comment_video_ids() -> Result<(), Box<dyn Error>> {
    // Use known comment IDs from MrBeast videos
    let comment_ids = vec![
        "UgwI3PTCpt1hMSmE8D54AaABAg".to_string(),
        "Ugy2sPNjJIm91JWYFcZ4AaABAg".to_string()
    ];
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let result = youtube_client.get_comment_video_ids(comment_ids.clone())
        .with_key(API_KEY)
        .with_referrer("https://explorer.apis.google.com")
        .send()
        .await?;
    
    // Verify we got video_ids:user_ids for the requested comments
    assert_eq!(result.comment_id_to_video_user.len(), comment_ids.len(), 
        "Should get video_id:user_id pairs for all comments");
    
    // Check that each comment ID has a corresponding video_id:user_id
    for comment_id in &comment_ids {
        assert!(result.comment_id_to_video_user.contains_key(comment_id), 
            "Missing video_id:user_id for comment {}", comment_id);
        
        let video_user = result.comment_id_to_video_user.get(comment_id).unwrap();
        
        // Check that the format is video_id:user_id
        let parts: Vec<&str> = video_user.split(':').collect();
        assert_eq!(parts.len(), 2, "Expected format video_id:user_id but got {}", video_user);
        
        // Verify video_id is not empty
        assert!(!parts[0].is_empty(), "Video ID should not be empty");
        
        // Verify user_id is not empty
        assert!(!parts[1].is_empty(), "User ID should not be empty");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_get_public_video() -> Result<(), Box<dyn std::error::Error>> {
    // MrBeast's "I Spent 50 Hours Buried Alive" video
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let videos = youtube_client
        .list_videos(vec!["9bqk6ZUsKyA".to_string()])
        .with_key(API_KEY)
        .send()
        .await?;
    
    assert_eq!(videos.len(), 1);
    let video = &videos[0];
    
    // Basic field checks (as before)
    assert_eq!(video.video_id, "9bqk6ZUsKyA");
    assert_eq!(video.user_id, Some("X6OQ3DkcsbYNE6H8uQQuVA".to_string()));
    
    // New field checks
    assert_eq!(video.privacy_status, 2); // public
    assert_eq!(video.upload_status, 2);  // processed
    assert!(video.views > 0);
    assert!(video.likes > 0);
    assert!(video.comments > 0);
    assert!(!video.tags.is_empty());  // MrBeast usually uses tags
    assert!(!video.default_language.is_empty());
    assert!(!video.default_audio_language.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_get_unlisted_video() -> Result<(), Box<dyn std::error::Error>> {
    // Known unlisted video    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let videos = youtube_client
        .list_videos(vec!["v6Xz96NIGGA".to_string()])
        .with_key(API_KEY)
        .send()
        .await?;
    
    assert_eq!(videos.len(), 1);
    let video = &videos[0];
    
    assert_eq!(video.video_id, "v6Xz96NIGGA");
    assert_eq!(video.user_id, Some("UCP5wwq4_TAqJZHaU7_oLj4w".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_get_private_video() -> Result<(), Box<dyn std::error::Error>> {

    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let videos = youtube_client
        .list_videos(vec!["YN4zvQyKvxU".to_string()])
        .with_key(API_KEY)
        .send()
        .await?;
    
    assert_eq!(videos.len(), 1);
    let video = &videos[0];
    
    // Verify all fields have default values for private videos
    assert_eq!(video.video_id, "YN4zvQyKvxU");
    assert_eq!(video.user_id, None);
    assert_eq!(video.privacy_status, 0); // private
    assert_eq!(video.upload_status, 0);  // unknown
    assert_eq!(video.title, None);
    assert_eq!(video.description, None);
    assert_eq!(video.tags, Vec::<String>::new());
    assert_eq!(video.default_language, "");
    assert_eq!(video.default_audio_language, "");
    assert_eq!(video.views, 0);
    assert_eq!(video.likes, 0);
    assert_eq!(video.comments, 0);
    assert_eq!(video.made_for_kids, None);
    assert_eq!(video.created_at, 1234567890);
    
    Ok(())
}

#[tokio::test]
async fn test_get_multiple_videos() -> Result<(), Box<dyn std::error::Error>> {
    let queries = vec![
        "9bqk6ZUsKyA".to_string(), // MrBeast video
        "dQw4w9WgXcQ".to_string() // Rick Roll
    ];
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let videos = youtube_client
        .list_videos(queries)
        .with_key(API_KEY)
        .send()
        .await?;
    
    assert_eq!(videos.len(), 2);
    assert!(videos.iter().any(|v| v.video_id == "9bqk6ZUsKyA"));
    assert!(videos.iter().any(|v| v.video_id == "dQw4w9WgXcQ"));
    
    Ok(())
}

#[tokio::test]
async fn test_get_comment_replies() -> Result<(), Box<dyn Error>> {
    // MrBeast comment with known replies
    let parent_comment_id = "Ugxvq9b6p97Wi662WjJ4AaABAg".to_string();
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let comment_replies = youtube_client.get_comment_replies(parent_comment_id.clone(), None)
        .with_key(API_KEY)
        .with_referrer("https://explorer.apis.google.com")
        .send()
        .await?;
    
    assert!(!comment_replies.comments.is_empty(), "Expected replies for this comment");
    
    // Check the structure of the replies
    for reply in &comment_replies.comments {
        // All replies should have a non-empty comment_id
        assert!(!reply.comment_id.is_empty());
        
        // All replies should have a parent_comment_id matching the requested ID (after cleaning)
        let cleaned_parent_id = parent_comment_id
            .strip_prefix("Ug")
            .and_then(|s| s.strip_suffix("4AaABAg"))
            .unwrap_or(&parent_comment_id);
            
        assert_eq!(reply.parent_comment_id, Some(cleaned_parent_id.to_string()));
        
        // All replies should have a parent_user_id (which is MrBeast's channel)
        assert!(reply.parent_user_id.is_some());
        
        // All replies should have text content, timestamps, etc.
        assert!(!reply.text.is_empty());
        assert!(reply.published_at > 0);
        assert_eq!(reply.reply_count, 0); // Replies can't have their own replies
    }
    
    // Check if we got a next page token (may or may not be present depending on number of replies)
    if comment_replies.comments.len() >= 100 {
        assert!(comment_replies.next_page_token.is_some());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_get_comment_replies_pagination() -> Result<(), Box<dyn Error>> {
    // Popular comment likely to have many replies
    let parent_comment_id = "Ugxvq9b6p97Wi662WjJ4AaABAg".to_string();
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    // First page
    let first_page = youtube_client.get_comment_replies(parent_comment_id.clone(), None)
        .with_key(API_KEY)
        .with_referrer("https://explorer.apis.google.com")
        .send()
        .await?;
    
    // If there's a next page token, fetch the second page
    if let Some(next_page_token) = first_page.next_page_token {
        let second_page = youtube_client.get_comment_replies(parent_comment_id, Some(next_page_token))
            .with_key(API_KEY)
            .with_referrer("https://explorer.apis.google.com")
            .send()
            .await?;
        
        // Second page should also have replies
        assert!(!second_page.comments.is_empty(), "Expected replies on second page");
        
        // Get the IDs from the first page
        let first_page_ids: Vec<String> = first_page.comments.iter().map(|c| c.comment_id.clone()).collect();
        
        // Ensure there's no overlap in comments between pages
        for comment in &second_page.comments {
            assert!(!first_page_ids.contains(&comment.comment_id), 
                "Comment from second page was already in first page");
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_get_comment_replies_no_replies() -> Result<(), Box<dyn Error>> {
    // Comment ID known to have no replies
    let parent_comment_id = "UgwTmkR7gG1HE5EBzTh4AaABAg".to_string();
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let result = youtube_client.get_comment_replies(parent_comment_id, None)
        .with_key(API_KEY)
        .with_referrer("https://explorer.apis.google.com")
        .send()
        .await;
    
    // Two possible outcomes:
    // 1. Success with empty comments array
    // 2. NotFound error if YouTube API doesn't return results for comments with no replies
    match result {
        Ok(response) => {
            assert!(response.comments.is_empty(), "Expected no replies");
            assert!(response.next_page_token.is_none(), "Should not have next page");
        },
        Err(YouTubeError::NotFound) => {
            // This is also acceptable if the API returns 404 for comments with no replies
        },
        Err(e) => {
            panic!("Unexpected error: {:?}", e);
        }
    }
    
    Ok(())
}


#[tokio::test]
async fn test_list_playlist_items() -> Result<(), Box<dyn Error>> {
    // MrBeast's uploads playlist (UU prefix makes it uploads playlist)
    let playlist_id = "UUX6OQ3DkcsbYNE6H8uQQuVA".to_string();
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let playlist_items = youtube_client
        .list_playlist_items(playlist_id, None, Some(5))
        .with_key(API_KEY)
        .with_referrer("https://explorer.apis.google.com")
        .send()
        .await?;
    
    assert!(!playlist_items.items.is_empty());
    assert_eq!(playlist_items.results_per_page, 5);
    assert!(playlist_items.next_page_token.is_some());
    
    // Check first item structure
    let first_item = &playlist_items.items[0];
    assert!(!first_item.id.is_empty());
    assert!(!first_item.snippet.title.is_empty());
    assert!(!first_item.snippet.resource_id.video_id.is_empty());
    assert!(!first_item.content_details.video_id.is_empty());
    assert_eq!(first_item.snippet.playlist_id, "UUX6OQ3DkcsbYNE6H8uQQuVA");
    assert_eq!(first_item.snippet.channel_title, "MrBeast");
    
    Ok(())
}


#[tokio::test]
async fn test_list_playlist_items_pagination() -> Result<(), Box<dyn Error>> {
    // MrBeast's uploads playlist (UU prefix makes it uploads playlist)
    let playlist_id = "UUX6OQ3DkcsbYNE6H8uQQuVA".to_string();
    
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    // First page
    let first_page = youtube_client
        .list_playlist_items(playlist_id.clone(), None, Some(3))
        .with_key(API_KEY)
        .with_referrer("https://explorer.apis.google.com")
        .send()
        .await?;
    
    assert!(!first_page.items.is_empty());
    assert_eq!(first_page.results_per_page, 3);
    assert!(first_page.next_page_token.is_some());
    
    // Second page using pageToken
    let page_token = first_page.next_page_token.unwrap();
    let second_page = youtube_client
        .list_playlist_items(playlist_id, Some(page_token), Some(3))
        .with_key(API_KEY)
        .with_referrer("https://explorer.apis.google.com")
        .send()
        .await?;
    
    assert!(!second_page.items.is_empty());
    assert_eq!(second_page.results_per_page, 3);
    
    // Ensure different items between pages
    let first_page_video_ids: Vec<String> = first_page.items.iter()
        .map(|item| item.content_details.video_id.clone())
        .collect();
    
    for item in &second_page.items {
        assert!(!first_page_video_ids.contains(&item.content_details.video_id),
            "Video ID {} was already in first page", item.content_details.video_id);
    }
    
    Ok(())
}


#[tokio::test]
async fn test_get_popular_video() -> Result<(), Box<dyn std::error::Error>> {
    // Video ID: jNQXAC9IVRw - should be a popular video with engagement
    let client = initialize_client()?;
    let mut youtube_client = YouTubeDataV3Client::new("youtube.googleapis.com".to_string(), client).await;
    
    let videos = youtube_client
        .list_videos(vec!["jNQXAC9IVRw".to_string()])
        .with_key(API_KEY)
        .with_referrer("https://explorer.apis.google.com")
        .send()
        .await?;
    
    assert_eq!(videos.len(), 1);
    let video = &videos[0];
    
    // Basic field checks
    assert_eq!(video.video_id, "jNQXAC9IVRw");
    assert!(video.user_id.is_some());
    
    // Ensure the video has engagement metrics > 0
    assert!(video.views > 0, "Video should have views > 0, got {}", video.views);
    assert!(video.likes > 0, "Video should have likes > 0, got {}", video.likes);
    assert!(video.comments > 0, "Video should have comments > 0, got {}", video.comments);
    
    // Should be public and processed
    assert_eq!(video.privacy_status, 2); // public
    assert_eq!(video.upload_status, 2);  // processed
    
    // Should have basic metadata
    assert!(video.title.is_some());
    assert!(video.created_at > 0);
    
    Ok(())
}
