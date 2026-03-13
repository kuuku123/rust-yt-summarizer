# YouTube Summarizer 🦀

A Rust application that retrieves videos from a specific YouTube channel from the last 24 hours, extracts transcripts, summarizes them using the Gemini AI API, and posts the summary to Telegram.

## Setup Instructions

1. **Obtain API Keys:**
   - **YouTube Data API v3:** Get from the [Google Cloud Console](https://console.cloud.google.com/).
   - **Gemini API Key:** Get from Google AI Studio.
   - **Telegram Bot Token:** Create a bot via `@BotFather` on Telegram.

2. **Configure Environment:**
   Copy `.env.example` to `.env` and fill in your keys:
   ```bash
   cp .env.example .env
   # Edit .env with your keys
   ```

3. **Get Telegram Chat ID:**
   You can send a message to your bot, then visit `https://api.telegram.org/bot<YOUR_BOT_TOKEN>/getUpdates` to find the `chat.id`.

4. **Get YouTube Channel ID:**
   Find the channel ID (usually starts with `UC...`) of the creator you want to summarize.

## Running the App Manually
To test that everything works:
```bash
cargo run
```

## Running Daily (Cron)
Since this app is designed to summarize the last 24 hours, you should set it to run once daily.
Open your crontab:
```bash
crontab -e
```
Add the following line to run it every day at 8:00 AM (modify paths accordingly):
```bash
0 8 * * * cd /home/tony/workspace/study/rust/yt-summarizer && /home/tony/.cargo/bin/cargo run >> /home/tony/workspace/study/rust/yt-summarizer/cron.log 2>&1
```
