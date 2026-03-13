use reqwest::Client;
use serde_json::json;
use std::error::Error;

pub struct GeminiClient {
    client: Client,
    api_key: String,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Summarizes the given text using the Gemini 2.5 Flash API
    pub async fn summarize_text(&self, text: &str) -> Result<String, Box<dyn Error>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            self.api_key
        );

        // A clear, structured prompt for technical and general videos
        let prompt = format!(
            "Please provide a concise, well-structured summary of the following video transcript. \
            Highlight the key takeaways in bullet points. Transcript:\n\n{}",
            text
        );

        let body = json!({
            "contents": [{
                "parts": [{"text": prompt}]
            }]
        });

        let response = self.client.post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        // Extracting just the text block from the response JSON
        let json_resp: serde_json::Value = response.json().await?;
        
        let summary_text = json_resp
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("Failed to extract summary from Gemini response.");

        Ok(summary_text.to_string())
    }
}
