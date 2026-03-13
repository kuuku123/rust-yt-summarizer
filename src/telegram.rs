use reqwest::Client;
use serde_json::json;
use std::error::Error;

pub struct TelegramClient {
    client: Client,
    bot_token: String,
}

impl TelegramClient {
    pub fn new(bot_token: String) -> Self {
        Self {
            client: Client::new(),
            bot_token,
        }
    }

    /// Sends a formatted message to the specified Telegram chat ID
    pub async fn send_message(&self, chat_id: &str, text: &str) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );

        let body = json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "Markdown",
            "disable_web_page_preview": true
        });

        let response = self.client.post(&url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Telegram API error: {}", error_text).into());
        }

        Ok(())
    }
}
