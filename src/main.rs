mod api;
mod config;
mod vocab;

use api::{Message, call_api};
use std::io;
use vocab::lemma::{Lemmatizer, NaiveLemmatizer};
use vocab::{VocabDb, VocabEntry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not found");

    let client = reqwest::Client::new();
    let mut history: Vec<Message> = Vec::new();

    println!("Was möchtest du üben? (What would you like to practice?)");
    let vocab_db = VocabDb::open("vocab.db")?;
    let lemmatizer = NaiveLemmatizer;

    let session_config = config::SessionConfig::from_cli();

    if session_config.verbose {
        println!("SessionConfig: {}", session_config);
    }
    let system_prompt = session_config.build_system_prompt();

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input_trimmed = input.trim();

        if input_trimmed.starts_with('/') {
            let parts: Vec<&str> = input_trimmed.splitn(3, ' ').collect();
            match parts[0] {
                "/quit" => {
                    println!("Tschüss!");
                    break;
                }
                "/help" => {
                    println!("/quit - Program beenden");
                    println!("/help - Befehle anzeigen");
                }
                "/vocab" => match parts.get(1).copied() {
                    Some("save") => match parts.get(2).copied() {
                        Some(word) => {
                            let context = history.last().map(|m| m.content.as_str()).unwrap_or("");
                            let entry = lemmatizer.lemmatize(parts[2], context);
                            let vocab_entry = VocabEntry {
                                lemma: entry.lemma,
                                pos: entry.pos,
                                source_form: word.to_string(),
                                translation: None,
                                context: Some(context.to_string()),
                            };
                            vocab_db.save(&vocab_entry)?;
                            println!("✓ Gespeichert: {}", vocab_entry.lemma);
                        }
                        None => println!("Usage: /vocab save <word>"),
                    },
                    Some("list") => {
                        let vocab_list = vocab_db.list()?;
                        if vocab_list.is_empty() {
                            println!("Keine Wörter gespeichert.")
                        } else {
                            println!("List: {:?}", vocab_list);
                        }
                    }
                    Some(other) => {
                        println!("Unknown /vocab command: {}", other);
                    }
                    None => println!("Usage: /vocab <save|list>"),
                },
                text => {
                    println!("Unknown command: {}", text);
                }
            }
        } else {
            let user_input = input.trim();
            println!("Du: {}", user_input);
            let message = Message::new("user", user_input);
            history.push(message);

            let response = call_api(&client, &api_key, &history, &system_prompt).await?;
            println!("Lehrer: {}", response);
            let message = Message::new("assistant", &response);
            history.push(message);
        }
    }
    Ok(())
}
