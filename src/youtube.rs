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

#[derive(Debug, Deserialize)]
pub struct VideoSnippet {
    pub title: String,
    pub description: String,
    #[serde(rename = "publishedAt")]
    pub published_at: String,
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

        let url = format!(
            "https://www.googleapis.com/youtube/v3/search?part=snippet&channelId={}&maxResults=10&order=date&type=video&publishedAfter={}&key={}",
            channel_id,
            published_after,
            self.api_key
        );

        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("YouTube API Error ({}): {}", status, error_text).into());
        }

        let search_response: YouTubeSearchResponse = response.json().await?;

        // Filter out any results that might not actually be videos (though type=video should handle this)
        // and ensure we only return items with a valid videoId.
        let videos = search_response.items.into_iter()
            .filter(|item| item.id.video_id.is_some())
            .collect();

        Ok(videos)
    }
}
