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
    name: CaptionName,
    #[serde(rename = "languageCode")]
    language_code: String,
}

#[derive(Debug, Deserialize)]
struct CaptionName {
    #[serde(rename = "simpleText")]
    simple_text: String,
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

        // Extract the ytInitialPlayerResponse JSON variable from the HTML
        let re = Regex::new(r#"var ytInitialPlayerResponse = (\{.*?\});var"#)?;
        let captures = re.captures(&html).ok_or("Could not find ytInitialPlayerResponse in HTML")?;
        
        let json_str = captures.get(1).unwrap().as_str();
        let player_response: Value = serde_json::from_str(json_str)?;

        // Navigate through the heavily nested JSON structure to find caption tracks
        let captions_block = player_response
            .get("captions")
            .and_then(|c| c.get("playerCaptionsTracklistRenderer"))
            .and_then(|p| p.get("captionTracks"))
            .ok_or("No captions available for this video.")?;

        let tracks: Vec<CaptionTrack> = serde_json::from_value(captions_block.clone())?;
        
        // Find English tracks preferred, or fallback to the first available
        let track = tracks.iter()
            .find(|t| t.language_code.starts_with("en"))
            .unwrap_or_else(|| tracks.first().unwrap());

        // The URL returns XML by default. We can add &fmt=json3 to get JSON, but XML is easy enough to parse manually.
        let transcript_xml = self.client.get(&track.base_url).send().await?.text().await?;
        
        // Very basic XML parser extracting <text> tags
        let text_re = Regex::new(r#"<text[^>]*>(.*?)</text>"#)?;
        let mut transcript = String::new();

        for cap in text_re.captures_iter(&transcript_xml) {
            let decoded = html_escape::decode_html_entities(cap.get(1).unwrap().as_str());
            transcript.push_str(&decoded);
            transcript.push(' ');
        }

        Ok(transcript.trim().to_string())
    }
}
