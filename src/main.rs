// Your task
// Update Cargo.toml with the four dependencies above. Then rewrite main.rs to:
//
// Load the API key from a .env file
// Define these structs (with appropriate derives):
//
// Message { role: String, content: String }
// ApiRequest { model: String, max_tokens: u32, system: String, messages: Vec<Message> }
// ContentBlock { text: String } — to deserialize the response
// ApiResponse { content: Vec<ContentBlock> } — the top-level response
//
//
// Write an async fn call_api(api_key: &str, user_input: &str) -> Result<String, Box<dyn std::error::Error>> that builds the request, sends it with reqwest, deserializes the response, and returns the text
// Call it from main, print the result

use std::io;

struct LemmaResult {
    lemma: String,
    pos: String,
    gender: Option<String>,
    translation: Option<String>,
}

#[derive(Debug)]
struct VocabEntry {
    lemma: String,
    pos: String,
    translation: Option<String>,
    source_form: String,
    context: Option<String>,
}

trait Lemmatizer {
    fn lemmatize(&self, word: &str, context: &str) -> LemmaResult;
}

struct NativeLemmatizer;

impl Lemmatizer for NativeLemmatizer {
    fn lemmatize(&self, word: &str, _context: &str) -> LemmaResult {
        LemmaResult {
            lemma: word.to_lowercase(),
            pos: "unknown".to_string(),
            gender: None,
            translation: None,
        }
    }
}

struct VocabDb {
    conn: rusqlite::Connection,
}

impl VocabDb {
    fn open(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute_batch(
            "
    CREATE TABLE IF NOT EXISTS vocab (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        lemma       TEXT NOT NULL,
        pos         TEXT NOT NULL,
        translation TEXT,
        source_form TEXT NOT NULL,
        context     TEXT,
        added_at    DATETIME DEFAULT CURRENT_TIMESTAMP,
        UNIQUE(lemma, pos)
    );
",
        )?;
        Ok(Self { conn })
    }

    fn save(&self, entry: &VocabEntry) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "INSERT OR IGNORE INTO vocab (lemma, pos, translation, source_form, context)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                entry.lemma,
                entry.pos,
                entry.translation,
                entry.source_form,
                entry.context
            ],
        )?;
        Ok(())
    }

    fn list(&self) -> Result<Vec<VocabEntry>, Box<dyn std::error::Error>> {
        let mut stmt = self
            .conn
            .prepare("SELECT lemma, pos, translation, source_form, context FROM vocab")?;
        let entries = stmt
            .query_map([], |row| {
                Ok(VocabEntry {
                    lemma: row.get(0)?,
                    pos: row.get(1)?,
                    translation: row.get(2)?,
                    source_form: row.get(3)?,
                    context: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok((entries))
    }
}

#[derive(serde::Serialize, Clone)]
struct Message {
    role: String,
    content: String,
}

impl Message {
    fn new(role: &str, content: &str) -> Message {
        Message {
            role: role.to_string(),
            content: content.to_string(),
        }
    }
}

#[derive(serde::Serialize)]
struct ApiRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: &'a [Message],
}

#[derive(serde::Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(serde::Deserialize)]
struct ApiResponse {
    content: Vec<ContentBlock>,
}

async fn call_api(
    client: &reqwest::Client,
    api_key: &str,
    history: &[Message],
) -> Result<String, Box<dyn std::error::Error>> {
    let request_body = ApiRequest {
        model: "claude-haiku-4-5-20251001",
        max_tokens: 1024,
        system: "Du bist ein freundlicher Deutschlehrer. \
                 Antworte immer auf Deutsch und korrigiere grammatikalische Fehler.",
        messages: history,
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request_body) // serializes your struct automatically
        .send()
        .await?
        .json::<ApiResponse>() // deserializes response into ApiResponse
        .await?;

    Ok(response
        .content
        .into_iter()
        .next()
        .ok_or("empty response")?
        .text)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not found");

    let client = reqwest::Client::new();
    let mut history: Vec<Message> = Vec::new();

    println!("Was möchtest du üben? (What would you like to practice?)");
    let vocab = VocabDb::open("vocab.db")?;
    let lemmatizer = NativeLemmatizer;

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
                            vocab.save(&vocab_entry)?;
                            println!("✓ Gespeichert: {}", vocab_entry.lemma);
                        }
                        None => println!("Usage: /vocab save <word>"),
                    },
                    Some("list") => {
                        let vocab_list = vocab.list()?;
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

            let response = call_api(&client, &api_key, &history).await?;
            println!("Lehrer: {}", response);
            let message = Message::new("assistant", &response);
            history.push(message);
        }
    }
    Ok(())
}
