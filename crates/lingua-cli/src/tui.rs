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

fn handle_help_command(app: &mut App) -> Result<(), LinguaError> {
    app.status = "/quit  /vocab save <word>  /vocab list  Esc=beenden".to_string();
    Ok(())
}

fn handle_vocab_save(app: &mut App, word: String) -> Result<(), LinguaError> {
    let context = app.history.last().map(|m| m.content.as_str()).unwrap_or("");
    let entry = app.lemmatizer.lemmatize(&word, context);
    let vocab_entry = VocabEntry {
        lemma: entry.lemma,
        pos: entry.pos,
        source_form: word.clone(),
        translation: None,
        context: Some(context.to_string()),
    };
    app.vocab_db.save(&vocab_entry)?;
    app.status = format!("✓ Gespeichert: {}", vocab_entry.lemma);
    Ok(())
}

fn handle_vocab_list(app: &mut App) -> Result<(), LinguaError> {
    let vocab_list = app.vocab_db.list()?;
    if vocab_list.is_empty() {
        app.status = format!("Keine Wörter gespeichert.")
    } else {
        let mut lines = format!("--- Vokcabelliste ({} Wörter) ---\n", vocab_list.len());
        for entry in &vocab_list {
            lines.push_str(&format!(
                "  {:<20} ({}) - {}\n",
                entry.lemma,
                entry.pos,
                entry.translation.as_deref().unwrap_or("_")
            ));
        }
        app.display_messages.push(Message::new("ferris", &lines));
        app.status = format!("{} Wörter geladen", vocab_list.len());
    }
    Ok(())
}

fn handle_send_message(app: &mut App, text: String) -> Result<(), LinguaError> {
    app.msg_requested = true;
    app.history.push(Message::new("user", &text));
    app.display_messages.push(Message::new("user", &text));
    app.status = "Warte auf Antwort...".to_string();
    Ok(())
}

fn handle_error_command(app: &mut App, error_msg: String) -> Result<(), LinguaError> {
    app.status = error_msg;
    Ok(())
}

enum CommandsKind {
    Quit,
    Help,
    VocabSave(String),
    VocabList,
    Message(String),
    Error(String),
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
    msg_requested: bool,
    pub scroll: usize,
    display_messages: Vec<Message>,
}

fn handle_commands(input: &str) -> CommandsKind {
    let input_trimmed = input.trim();

    if input_trimmed.starts_with('/') {
        let parts: Vec<&str> = input_trimmed.splitn(3, ' ').collect();
        match parts[0] {
            "/quit" => CommandsKind::Quit,
            "/help" => CommandsKind::Help,
            "/vocab" => match parts.get(1).copied() {
                Some("save") => match parts.get(2).copied() {
                    Some(word) => CommandsKind::VocabSave(word.to_string()),
                    None => CommandsKind::Error("Usage: /vocab save <word>".to_string()),
                },
                Some("list") => CommandsKind::VocabList,
                Some(other) => CommandsKind::Error(format!("Unknown /vocab command: {}", other)),
                None => CommandsKind::Error("Usage: /vocab <save|list>".to_string()),
            },
            text => CommandsKind::Error(format!("Unknown command: {}", text)),
        }
    } else {
        CommandsKind::Message(input_trimmed.to_string())
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

    let display_lines: Vec<Line> = app
        .display_messages
        .iter()
        .flat_map(|msg| {
            let (label, color) = match msg.role.as_str() {
                "user" => ("Du:    ", Color::Green),
                "assistant" => ("Lehrer:    ", Color::Cyan),
                "ferris" => ("Ferris: ", Color::Yellow),
                _ => ("    ", Color::White),
            };

            msg.content
                .lines()
                .enumerate()
                .map(|(i, line)| {
                    let prefix = if i == 0 { label } else { "    " };
                    Line::from(vec![
                        Span::styled(
                            prefix,
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(line.to_string()),
                    ])
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let chat_widget = Paragraph::new(display_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Lingua Ferris :crab: "),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll as u16, 0));
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
        msg_requested: false,
        scroll: 0,
        display_messages: Vec::new(),
    };

    let (tx, mut rx) = mpsc::channel::<Result<String, LinguaError>>(1);
    loop {
        terminal.draw(|frame| {
            draw(frame, &app);
        })?;

        if app.msg_requested {
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
            app.msg_requested = false;
            app.waiting = true;
        }

        if app.waiting {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(response) => {
                        app.history.push(Message::new("assistant", &response));
                        app.display_messages
                            .push(Message::new("assistant", &response));
                        app.scroll = app.display_messages.len().saturating_sub(1);
                        app.status.clear();
                    }
                    Err(e) => {
                        app.status = format!("Error: {}", e);
                    }
                }
                app.waiting = false;
            }
        }

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    crossterm::event::KeyCode::Esc => app.should_quit = true,
                    crossterm::event::KeyCode::Enter => {
                        if !app.waiting && !app.input.is_empty() {
                            let text = app.input.clone();
                            match handle_commands(&text) {
                                CommandsKind::Quit => app.should_quit = true,
                                CommandsKind::VocabSave(word) => handle_vocab_save(&mut app, word)?,
                                CommandsKind::VocabList => handle_vocab_list(&mut app)?,
                                CommandsKind::Help => handle_help_command(&mut app)?,
                                CommandsKind::Message(text) => handle_send_message(&mut app, text)?,
                                CommandsKind::Error(error_msg) => {
                                    handle_error_command(&mut app, error_msg)?
                                }
                            }
                            app.input.clear();
                        }
                    }
                    crossterm::event::KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    crossterm::event::KeyCode::Backspace => {
                        app.input.pop();
                    }
                    crossterm::event::KeyCode::Up => {
                        app.scroll = app.scroll.saturating_sub(1);
                    }
                    crossterm::event::KeyCode::Down => {
                        app.scroll += 1;
                    }
                    _ => {}
                }
                if app.should_quit {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    Ok(())
}
