use clap::Parser;
use std::fmt;

#[derive(Clone)]
pub struct SessionConfig {
    pub level: String,
    pub topic: String,
    pub max_tokens: u32,
    pub verbose: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            level: "b2".to_string(),
            topic: "general".to_string(),
            max_tokens: 1024,
            verbose: false,
        }
    }
}

impl fmt::Display for SessionConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Level: {} | Topic: {} | Max tokens: {}",
            self.level, self.topic, self.max_tokens
        )
    }
}

impl SessionConfig {
    pub fn from_cli() -> Self {
        let cli = Cli::parse();
        Self {
            level: cli.level,
            topic: cli.topic,
            max_tokens: cli.max_tokens,
            verbose: cli.verbose,
        }
    }
    pub fn build_system_prompt(&self) -> String {
        format!(
            "Du bist ein freundlicher Deutschlehrer für Niveau {}. \
            Antworte immer auf Deutsch und korrigiere grammatikalische Fehler. \
            Das aktuelle Thema ist: {}.",
            self.level.to_uppercase(),
            self.topic
        )
    }
}

#[derive(Parser)]
#[command(name = "lingua-ferris", about = "Dein Deutschlehrer")]
struct Cli {
    #[arg(long, default_value = "b2")]
    level: String,

    #[arg(long, default_value = "general")]
    topic: String,

    #[arg(long, default_value_t = 1024)]
    max_tokens: u32,

    #[arg(long)]
    verbose: bool,
}
