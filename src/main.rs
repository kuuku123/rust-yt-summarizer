mod youtube;
mod transcript;
mod gemini;
mod telegram;

use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Load environment variables
    let yt_api_key = env::var("YOUTUBE_API_KEY").expect("YOUTUBE_API_KEY must be set");
    let channel_id = env::var("YOUTUBE_CHANNEL_ID").expect("YOUTUBE_CHANNEL_ID must be set");
    let gemini_api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let telegram_token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
    let telegram_chat_id = env::var("TELEGRAM_CHAT_ID").expect("TELEGRAM_CHAT_ID must be set");

    println!("Checking for new videos on YouTube channel: {}", channel_id);

    // 1. Fetch recent videos
    let yt_client = youtube::YouTubeClient::new(yt_api_key);
    let videos = match yt_client.get_recent_videos(&channel_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to fetch videos from YouTube: {}", e);
            return Err(e);
        }
    };

    if videos.is_empty() {
        println!("No new videos found in the last 24 hours.");
        return Ok(());
    }

    println!("Found {} new video(s). Processing...", videos.len());

    let transcript_client = transcript::TranscriptClient::new();
    let gemini_client = gemini::GeminiClient::new(gemini_api_key);
    let telegram_client = telegram::TelegramClient::new(telegram_token);

    // 2. Process each video
    for video in videos {
        if let Some(video_id) = video.id.video_id {
            println!("- Processing video: '{}' (ID: {})", video.snippet.title, video_id);
            
            // 3. Get transcript
            let transcript_text = match transcript_client.get_transcript(&video_id).await {
                Ok(text) => text,
                Err(e) => {
                    eprintln!("  -> Could not fetch transcript: {}", e);
                    continue; // Skip to next video if transcript fails
                }
            };

            if transcript_text.is_empty() {
                println!("  -> Transcript was empty. Skipping.");
                continue;
            }

            println!("  -> Transcript fetched ({} chars). Summarizing...", transcript_text.len());

            // 4. Summarize with Gemini
            let summary = match gemini_client.summarize_text(&transcript_text).await {
                Ok(text) => text,
                Err(e) => {
                    eprintln!("  -> Failed to summarize with Gemini: {}", e);
                    continue;
                }
            };

            // 5. Send to Telegram
            let tg_message = format!(
                "🎥 *New Video Summary*\n\n*Title:* {}\n*Link:* https://youtube.com/watch?v={}\n\n*Summary:*\n{}",
                video.snippet.title,
                video_id,
                summary
            );

            match telegram_client.send_message(&telegram_chat_id, &tg_message).await {
                Ok(_) => println!("  -> Successfully sent summary to Telegram!"),
                Err(e) => eprintln!("  -> Failed to send to Telegram: {}", e),
            }
        }
    }

    println!("All processing complete.");
    Ok(())
}
