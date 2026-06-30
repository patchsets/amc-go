use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io::stdout;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

pub struct ResultMsg {
    pub content: String,
    pub capture: String,
    pub timestamp: String,
}

impl ResultMsg {
    pub fn display(&self) -> String {
        if !self.content.is_empty() {
            format!("[{}] {} | {}", self.timestamp, self.content, self.capture)
        } else {
            "..................................................".into()
        }
    }
}

pub enum UiMessage {
    Result(ResultMsg),
    Done,
}

const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

pub fn run_tui(rx: mpsc::Receiver<UiMessage>, quit: Arc<AtomicBool>) {
    let _ = enable_raw_mode();
    let _ = stdout().execute(EnterAlternateScreen);

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).expect("failed to create terminal");

    let mut results: Vec<String> = vec!["..................................................".into(); 10];
    let mut spinner_idx: usize = 0;
    let mut tick: u64 = 0;
    let mut done = false;

    let banner = r#"  _____ __        ______ __                __
 / ___// /____   / / / /___ ______       /  |  /  /  ___
  \__ \/ __/ _ \/ / / __  / ___/       / /| | / /|/| / / /
 ___/ / /_/  __/ / / /_/ / /          / ___ |/ / / |/ / /___
/____/\__/\___/_/_/\__,_/_/          /_/  |_/_/  |_/\____/"#;

    loop {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                UiMessage::Result(r) => {
                    results.push(r.display());
                    if results.len() > 10 {
                        results.remove(0);
                    }
                }
                UiMessage::Done => {
                    done = true;
                }
            }
        }

        let spinner_char = SPINNER_CHARS[spinner_idx % SPINNER_CHARS.len()];
        tick += 1;
        if tick % 2 == 0 {
            spinner_idx += 1;
        }

        let _ = terminal.draw(|f| {
            let area = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(7),
                    Constraint::Length(2),
                    Constraint::Min(10),
                    Constraint::Length(2),
                ])
                .split(area);

            let banner_widget = Paragraph::new(banner)
                .style(Style::default().fg(Color::Rgb(120, 60, 170)));
            f.render_widget(banner_widget, chunks[0]);

            let status = if done {
                "Checking complete!".to_string()
            } else {
                format!("{} Running Checker...", spinner_char)
            };
            let status_widget = Paragraph::new(status)
                .style(Style::default().fg(Color::Rgb(5, 189, 245)));
            f.render_widget(status_widget, chunks[1]);

            let items: Vec<ListItem> = results
                .iter()
                .map(|r| {
                    let style = if r.contains("Invalid") {
                        Style::default().fg(Color::Red)
                    } else if r.contains("Valid Account") && r.contains("Balance") {
                        Style::default().fg(Color::Green)
                    } else if r.contains("Valid Account") {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    ListItem::new(r.as_str()).style(style)
                })
                .collect();
            let results_list = List::new(items)
                .block(Block::default().borders(Borders::TOP).title("Results"));
            f.render_widget(results_list, chunks[2]);

            let help = if done {
                "Press any key to exit"
            } else {
                "Press q or Ctrl+C to quit"
            };
            let help_widget = Paragraph::new(help)
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(help_widget, chunks[3]);
        });

        if event::poll(Duration::from_millis(100)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                    if done {
                        break;
                    }
                    if key.code == KeyCode::Char('q')
                        || key.code == KeyCode::Char('c')
                            && key.modifiers.contains(event::KeyModifiers::CONTROL)
                    {
                        quit.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            }
        }

        if done && quit.load(Ordering::Relaxed) {
            break;
        }
    }

    let _ = disable_raw_mode();
    let _ = stdout().execute(LeaveAlternateScreen);
}
