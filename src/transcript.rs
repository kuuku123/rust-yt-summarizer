use reqwest::Client;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;

pub struct TranscriptClient {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct CaptionTrack {
    #[serde(rename = "baseUrl")]
    base_url: String,
    #[serde(rename = "languageCode")]
    language_code: String,
}

impl TranscriptClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36")
                .build()
                .unwrap(),
        }
    }

    pub async fn get_transcript(&self, video_id: &str) -> Result<String, Box<dyn Error>> {
        let url = format!("https://www.youtube.com/watch?v={}", video_id);
        let html = self.client.get(&url).send().await?.text().await?;

        // 1. Extract the INNERTUBE_API_KEY and ytInitialPlayerResponse
        let key_re = Regex::new(r#"\"INNERTUBE_API_KEY\":\s*\"([^\"]+)\""#)?;
        let captures = key_re.captures(&html).ok_or("Could not find INNERTUBE_API_KEY in HTML")?;
        let api_key = captures.get(1).unwrap().as_str();

        // 2. Fetch fresh tracks from innertube API (this bypasses 'exp=xpe' proof-of-token blocks)
        let innertube_url = format!("https://www.youtube.com/youtubei/v1/player?key={}", api_key);
        
        let payload = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "ANDROID",
                    "clientVersion": "20.10.38"
                }
            },
            "videoId": video_id
        });

        let innertube_response: Value = self.client
            .post(&innertube_url)
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;

        // 3. Navigate the heavily nested JSON
        let captions_block = innertube_response
            .get("captions")
            .and_then(|c| c.get("playerCaptionsTracklistRenderer"))
            .and_then(|p| p.get("captionTracks"))
            .ok_or_else(|| {
                // Return a clear error with the actual response JSON to help debug
                match serde_json::to_string_pretty(&innertube_response) {
                    Ok(json_str) => format!("No captions available for this video.\nResponse: {}", json_str),
                    Err(_) => "No captions available for this video.".to_string(),
                }
            })?;

        let tracks: Vec<CaptionTrack> = serde_json::from_value(captions_block.clone())?;
        
        // Find Korean tracks preferred, or fallback to the first available
        let track = tracks.iter()
            .find(|t| t.language_code.starts_with("ko"))
            .unwrap_or_else(|| tracks.first().unwrap());

        // 4. Download and parse the Android default XML transcript format
        let transcript_xml = self.client.get(&track.base_url).send().await?.text().await?;
        
        let tags_re = Regex::new(r#"<[^>]*>"#)?;
        
        // Strip all XML tags, replacing them with a space so words don't get glued together
        let mut transcript = tags_re.replace_all(&transcript_xml, " ").into_owned();
        
        // Decode HTML entities (e.g., &#39; to ')
        transcript = html_escape::decode_html_entities(&transcript).into_owned();
        
        // Normalize whitespace (remove multiple spaces and newlines)
        let whitespace_re = Regex::new(r#"\s+"#)?;
        transcript = whitespace_re.replace_all(&transcript, " ").into_owned();

        Ok(transcript.trim().to_string())
    }
}
