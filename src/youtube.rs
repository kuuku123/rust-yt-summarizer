use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use chrono::{Utc, Duration, DateTime};

#[derive(Debug, Deserialize)]
pub struct YouTubeSearchResponse {
    pub items: Vec<YouTubeSearchResult>,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeSearchResult {
    pub id: VideoId,
    pub snippet: VideoSnippet,
}

#[derive(Debug, Deserialize)]
pub struct VideoId {
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct VideoSnippet {
    pub title: String,
    pub description: String,
    #[serde(rename = "publishedAt")]
    pub published_at: String,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeVideosResponse {
    pub items: Vec<YouTubeVideoDetails>,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeVideoDetails {
    pub id: String,
    #[serde(rename = "liveStreamingDetails")]
    pub live_streaming_details: Option<serde_json::Value>,
}

pub struct YouTubeClient {
    client: Client,
    api_key: String,
}

impl YouTubeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Fetches videos from the specified channel published within the last 24 hours.
    pub async fn get_recent_videos(&self, channel_id: &str) -> Result<Vec<YouTubeSearchResult>, Box<dyn Error>> {
        let one_day_ago: DateTime<Utc> = Utc::now() - Duration::days(1);
        let published_after = one_day_ago.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

        let search_url = format!(
            "https://www.googleapis.com/youtube/v3/search?part=snippet&channelId={}&maxResults=10&order=date&type=video&publishedAfter={}&key={}",
            channel_id,
            published_after,
            self.api_key
        );

        let search_response = self.client.get(&search_url).send().await?;
        
        if !search_response.status().is_success() {
            let status = search_response.status();
            let error_text = search_response.text().await?;
            return Err(format!("YouTube Search API Error ({}): {}", status, error_text).into());
        }

        let search_data: YouTubeSearchResponse = search_response.json().await?;

        // Extract valid video IDs from the search response
        let video_ids: Vec<String> = search_data.items.iter()
            .filter_map(|item| item.id.video_id.clone())
            .collect();

        if video_ids.is_empty() {
            return Ok(vec![]);
        }

        // Make a second API call to get video details (specifically liveStreamingDetails)
        let ids_csv = video_ids.join(",");
        let details_url = format!(
            "https://www.googleapis.com/youtube/v3/videos?part=liveStreamingDetails&id={}&key={}",
            ids_csv,
            self.api_key
        );

        let details_response = self.client.get(&details_url).send().await?;
        if !details_response.status().is_success() {
            let status = details_response.status();
            let error_text = details_response.text().await?;
            return Err(format!("YouTube Videos API Error ({}): {}", status, error_text).into());
        }

        let details_data: YouTubeVideosResponse = details_response.json().await?;

        // Create a list of IDs that are NOT live streams
        let standard_video_ids: Vec<String> = details_data.items.into_iter()
            .filter(|item| item.live_streaming_details.is_none()) // If it has live details, it's a stream/VOD
            .map(|item| item.id)
            .collect();

        // Filter the original search results to only include standard videos
        let filtered_videos = search_data.items.into_iter()
            .filter(|item| {
                if let Some(vid) = &item.id.video_id {
                    standard_video_ids.contains(vid)
                } else {
                    false
                }
            })
            .collect();

        Ok(filtered_videos)
    }
}
