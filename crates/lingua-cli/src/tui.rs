use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use dotenvy::Error;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Terminal,
};

use lingua_core::{
    call_api_with_retry, Lemmatizer, LinguaError, Message, NaiveLemmatizer, SessionConfig, VocabDb,
    VocabEntry,
};

pub trait Command {
    fn execute(&mut self, app: &mut App) -> Result<(), LinguaError>;
}

pub struct QuitCommand;
pub struct HelpCommand;
pub struct VocabSaveCommand {
    pub word: String,
}
pub struct VocabListCommand;
pub struct SendMessageCommand {
    pub text: String,
}
pub struct ErrorCommand {
    pub error_msg: String,
}

impl Command for QuitCommand {
    fn execute(&mut self, app: &mut App) -> Result<(), LinguaError> {
        app.should_quit = true;
        Ok(())
    }
}

impl Command for HelpCommand {
    fn execute(&mut self, app: &mut App) -> Result<(), LinguaError> {
        println!("/quit - Program beenden");
        println!("/help - Befehle anzeigen");
        Ok(())
    }
}

impl Command for VocabSaveCommand {
    fn execute(&mut self, app: &mut App) -> Result<(), LinguaError> {
        let context = app.history.last().map(|m| m.content.as_str()).unwrap_or("");
        let entry = app.lemmatizer.lemmatize(&self.word, context);
        let vocab_entry = VocabEntry {
            lemma: entry.lemma,
            pos: entry.pos,
            source_form: self.word.clone(),
            translation: None,
            context: Some(context.to_string()),
        };
        app.vocab_db.save(&vocab_entry)?;
        app.status = format!("✓ Gespeichert: {}", vocab_entry.lemma);
        Ok(())
    }
}

impl Command for VocabListCommand {
    fn execute(&mut self, app: &mut App) -> Result<(), LinguaError> {
        let vocab_list = app.vocab_db.list()?;
        if vocab_list.is_empty() {
            app.status = format!("Keine Wörter gespeichert.")
        } else {
            app.status = format!("\n--- Vokcabelliste ({} Wörter) ---", vocab_list.len());
            for entry in &vocab_list {
                println!(
                    "  {:<20} ({}) - {}",
                    entry.lemma,
                    entry.pos,
                    entry.translation.as_deref().unwrap_or("_")
                );
            }
        }
        Ok(())
    }
}

impl Command for SendMessageCommand {
    fn execute(&mut self, app: &mut App) -> Result<(), LinguaError> {
        // Fallthrough, could be a useful hook.
        Ok(())
    }
}

impl Command for ErrorCommand {
    fn execute(&mut self, app: &mut App) -> Result<(), LinguaError> {
        // Fallthrough, could be a useful hook.
        Ok(())
    }
}

enum Commands {
    Quit,
    Help,
    VocabSave(String),
    VocabLlist,
    Message(String),
}

use tokio::sync::mpsc;

pub struct App {
    history: Vec<Message>,
    input: String,
    vocab_db: VocabDb,
    lemmatizer: NaiveLemmatizer,
    config: SessionConfig,
    system_prompt: String,
    status: String,
    waiting: bool,
    api_key: String,
    client: reqwest::Client,
    should_quit: bool,
}

fn handle_commands(input: &str) -> Box<dyn Command> {
    let input_trimmed = input.trim();

    if input_trimmed.starts_with('/') {
        let parts: Vec<&str> = input_trimmed.splitn(3, ' ').collect();
        match parts[0] {
            "/quit" => Box::new(QuitCommand),
            "/help" => Box::new(HelpCommand),
            "/vocab" => match parts.get(1).copied() {
                Some("save") => match parts.get(2).copied() {
                    Some(word) => Box::new(VocabSaveCommand {
                        word: word.to_string(),
                    }),
                    None => Box::new(ErrorCommand {
                        error_msg: "Usage: /vocab save <word>".to_string(),
                    }),
                },
                Some("list") => Box::new(VocabListCommand),
                Some(other) => Box::new(ErrorCommand {
                    error_msg: format!("Unknown /vocab command: {}", other),
                }),
                None => Box::new(ErrorCommand {
                    error_msg: "Usage: /vocab <save|list>".to_string(),
                }),
            },
            text => Box::new(ErrorCommand {
                error_msg: format!("Unknown command: {}", text),
            }),
        }
    } else {
        Box::new(SendMessageCommand {
            text: input_trimmed.to_string(),
        })
    }
}

fn draw(frame: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let history_lines: Vec<Line> = app
        .history
        .iter()
        .map(|msg| {
            let (label, color) = match msg.role.as_str() {
                "user" => ("Du:    ", Color::Green),
                "assistant" => ("Lehrer:    ", Color::Cyan),
                _ => ("system:    ", Color::White),
            };
            Line::from(vec![
                Span::styled(
                    label,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(msg.content.clone()),
            ])
        })
        .collect();

    let chat_widget = Paragraph::new(history_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Lingua Ferris :crab: "),
        )
        .wrap(Wrap { trim: false });
    // .scroll((app.scroll as u16, 0));

    let input_widget = Paragraph::new(app.input.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Eingabe ")
            .border_style(Style::default().fg(Color::Yellow)),
    );

    let status_widget =
        Paragraph::new(app.status.as_str()).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(chat_widget, chunks[0]);
    frame.render_widget(input_widget, chunks[1]);
    frame.render_widget(status_widget, chunks[2]);
}

pub async fn run(
    api_key: String,
    config: SessionConfig,
    vocab_db: VocabDb,
    client: reqwest::Client,
) -> Result<(), LinguaError> {
    // setup terminal
    // create App
    // event loop
    // teardown terminal

    // setup
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = std::io::stdout().execute(LeaveAlternateScreen);
        original_hook(panic_info);
    }));
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let system_prompt = config.build_system_prompt();

    // create App
    let mut app = App {
        history: Vec::new(),
        input: String::new(),
        vocab_db,
        lemmatizer: NaiveLemmatizer,
        config,
        system_prompt,
        status: String::new(),
        waiting: false,
        api_key,
        client,
        should_quit: false,
    };

    let (tx, mut rx) = mpsc::channel::<Result<String, LinguaError>>(1);
    let mut msg_requested = false;
    loop {
        terminal.draw(|frame| {
            draw(frame, &app);
        })?;

        if app.waiting && msg_requested {
            let api_key_clone = app.api_key.clone();
            let history_clone = app.history.clone();
            let system_prompt_clone = app.system_prompt.clone();
            let tx_clone = tx.clone();
            let client_clone = app.client.clone();
            tokio::spawn(async move {
                let result = call_api_with_retry(
                    &client_clone,
                    &api_key_clone,
                    &history_clone,
                    &system_prompt_clone,
                )
                .await;
                tx_clone.send(result).await.ok();
            });
            msg_requested = false;
        }

        if app.waiting && !msg_requested {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(response) => {
                        app.history.push(Message::new("assistant", &response));
                    }
                    Err(e) => {
                        app.status = format!("Error: {}", e);
                    }
                }
                app.waiting = false;
                app.status.clear();
            }
        }

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    crossterm::event::KeyCode::Esc => break,
                    crossterm::event::KeyCode::Enter => {
                        if !app.waiting {
                            msg_requested = true;
                            app.waiting = true;
                            let text = app.input.clone();
                            app.history.push(Message::new("user", &text));
                            app.input.clear();
                            app.status = "Warte auf Antwort...".to_string();
                        }
                    }
                    crossterm::event::KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    crossterm::event::KeyCode::Backspace => {
                        app.input.pop();
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    Ok(())
}
