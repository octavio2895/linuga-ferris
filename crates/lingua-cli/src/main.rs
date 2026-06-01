use lingua_core::{
    call_api_with_retry, LinguaError, Message, NaiveLemmatizer, SessionConfig, VocabDb,
};

mod tui;

#[tokio::main]
async fn main() -> Result<(), LinguaError> {
    dotenvy::dotenv().ok();
    let api_key = std::env::var("ANTHROPIC_API_KEY")?;
    let config = SessionConfig::from_cli();
    let vocab_db = VocabDb::open("vocab.db")?;
    let client = reqwest::Client::new();

    tui::run(api_key, config, vocab_db, client).await
}
