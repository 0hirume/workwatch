use std::{env, io, time::Duration};

use chrono::Local;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use dotenv::dotenv;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};
use reqwest::Client;
use serde_json::json;
use tui_input::{Input, backend::crossterm::EventHandler};

enum AppState {
    Menu,
    Working,
    Logs,
}

#[derive(PartialEq, Eq)]
enum PromptState {
    Input,
    Edit,
    NoPrompt,
}

pub struct WorkWatcherApp {
    state: AppState,
    time: usize,
    logs: Vec<String>,
    prompt_state: PromptState,
    prompt_input: Input,
    selected_log: Option<usize>,
    client: Client,
    username: String,
    webhook_url: String,
    bot_name: String,
}

impl WorkWatcherApp {
    pub fn new(username: String, webhook_url: String) -> Self {
        WorkWatcherApp {
            state: AppState::Menu,
            time: 0,
            logs: vec![],
            prompt_state: PromptState::NoPrompt,
            prompt_input: Input::default(),
            selected_log: None,
            client: Client::new(),
            username,
            webhook_url,
            bot_name: "WorkWatch".to_string(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut terminal = ratatui::init();

        loop {
            terminal.draw(|frame| {
                self.draw(frame);
            })?;

            if event::poll(Duration::from_secs(1))? {
                let key_event = event::read()?;

                if let Event::Key(key) = key_event {
                    if key.kind == KeyEventKind::Release {
                        continue;
                    }

                    match self.prompt_state {
                        PromptState::Input => {
                            self.prompt_input.handle_event(&key_event);

                            match key.code {
                                KeyCode::Enter => {
                                    self.logs.push(self.prompt_input.value_and_reset());

                                    if self.selected_log.is_none() {
                                        self.selected_log = Some(0);
                                    }

                                    self.prompt_state = PromptState::NoPrompt;
                                }
                                KeyCode::Esc => {
                                    self.prompt_input.reset();
                                    self.prompt_state = PromptState::NoPrompt;
                                }
                                _ => {}
                            }

                            continue;
                        }
                        PromptState::Edit => {
                            self.prompt_input.handle_event(&key_event);

                            match key.code {
                                KeyCode::Enter => {
                                    if let Some(index) = self.selected_log {
                                        self.logs[index] = self.prompt_input.value_and_reset();
                                    }

                                    self.prompt_state = PromptState::NoPrompt;
                                }
                                KeyCode::Esc => {
                                    self.prompt_input.reset();
                                    self.prompt_state = PromptState::NoPrompt;
                                }
                                _ => {}
                            }

                            continue;
                        }
                        PromptState::NoPrompt => {}
                    }

                    match self.state {
                        AppState::Menu => match key.code {
                            KeyCode::Char('c') => {
                                self.state = AppState::Working;
                                self.send_clock_in_webhook();
                                self.time = 0;
                            }
                            KeyCode::Char('q') => break,
                            _ => {}
                        },
                        AppState::Working => match key.code {
                            KeyCode::Char('c') => {
                                self.state = AppState::Menu;
                                self.send_clock_out_webhook();
                                self.time = 0;
                            }
                            KeyCode::Char('a') => {
                                self.prompt_state = PromptState::Input;
                            }
                            KeyCode::Char('l') => {
                                self.state = AppState::Logs;
                            }
                            _ => {}
                        },
                        AppState::Logs => match key.code {
                            KeyCode::Char('t') => {
                                self.state = AppState::Working;
                            }
                            KeyCode::Char('a') => {
                                self.prompt_state = PromptState::Input;
                            }
                            KeyCode::Char('e') => {
                                if let Some(index) = self.selected_log {
                                    self.prompt_input = self.logs[index].clone().into();
                                    self.prompt_state = PromptState::Edit;
                                }
                            }
                            KeyCode::Char('d') => {
                                if let Some(index) = self.selected_log {
                                    self.logs.remove(index);
                                    if self.logs.is_empty() {
                                        self.selected_log = None;
                                    } else {
                                        self.selected_log =
                                            Some(index.saturating_sub(1).min(self.logs.len() - 1));
                                    }
                                }
                            }
                            KeyCode::Char('c') => {
                                self.state = AppState::Menu;
                                self.send_clock_out_webhook();
                                self.time = 0;
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if self.prompt_state != PromptState::Edit {
                                    if let Some(index) = self.selected_log {
                                        let len = self.logs.len();
                                        self.selected_log = Some((index + len - 1) % len);
                                    }
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if self.prompt_state != PromptState::Edit {
                                    if let Some(index) = self.selected_log {
                                        let len = self.logs.len();
                                        self.selected_log = Some((index + 1) % len);
                                    }
                                }
                            }
                            _ => {}
                        },
                    }
                }
            } else if let AppState::Working = self.state {
                self.time = self.time.saturating_add(1);
            }
        }

        ratatui::restore();

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let title = match self.state {
            AppState::Menu => "Menu",
            AppState::Working => "Working",
            AppState::Logs => "Logs",
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(match self.prompt_state {
                PromptState::NoPrompt => vec![Constraint::Min(0), Constraint::Length(3)],
                _ => vec![
                    Constraint::Min(0),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ],
            })
            .split(area);

        frame.render_widget(
            match self.state {
                AppState::Menu => Paragraph::new(vec![Line::from(format!(
                    "Welcome To WorkWatch, {}",
                    self.username
                ))]),
                AppState::Working => Paragraph::new(vec![Line::from(format!(
                    "Elapsed Time: {}",
                    self.get_compact_time()
                ))]),
                AppState::Logs => Paragraph::new(if self.logs.is_empty() {
                    vec![Line::from("No Logs Yet")]
                } else {
                    self.logs
                        .iter()
                        .enumerate()
                        .map(|(index, log)| {
                            if Some(index) == self.selected_log {
                                Line::from(Span::styled(
                                    log.as_str(),
                                    Style::new()
                                        .fg(Color::LightGreen)
                                        .add_modifier(Modifier::BOLD),
                                ))
                            } else {
                                Line::from(log.as_str())
                            }
                        })
                        .collect::<Vec<Line>>()
                }),
            }
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(title),
            )
            .alignment(Alignment::Center),
            chunks[0],
        );

        match self.prompt_state {
            PromptState::Input => {
                frame.render_widget(
                    Paragraph::new(self.prompt_input.to_string()).block(
                        Block::bordered()
                            .border_type(BorderType::Rounded)
                            .title("Input"),
                    ),
                    chunks[1],
                );
            }
            PromptState::Edit => {
                frame.render_widget(
                    Paragraph::new(self.prompt_input.to_string()).block(
                        Block::bordered()
                            .border_type(BorderType::Rounded)
                            .title("Edit"),
                    ),
                    chunks[1],
                );
            }
            PromptState::NoPrompt => {}
        }

        frame.render_widget(
            match self.state {
                AppState::Menu => Paragraph::new(vec![Line::from(" C - Clock In | Q - Quit ")]),
                AppState::Working => Paragraph::new(vec![Line::from(
                    " L - View Logs | A - Add Log | C - Clock Out ",
                )]),
                AppState::Logs => Paragraph::new(vec![Line::from(
                    " T - View Time | A - Add Log | E - Edit Log | D - Delete Log | C - Clock Out ",
                )]),
            }
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title("Controls"),
            ),
            chunks[match self.prompt_state {
                PromptState::NoPrompt => 1,
                _ => 2,
            }],
        );
    }

    fn send_clock_in_webhook(&self) {
        if self.webhook_url.is_empty() {
            return;
        }

        let client = self.client.clone();
        let webhook_url = self.webhook_url.clone();
        let bot_name = self.bot_name.clone();
        let username = self.username.clone();

        tokio::spawn(async move {
            let title = format!("{} has clocked in!", username);
            let now = Local::now();
            let date = now.format("%m/%d/%Y").to_string();
            let time = now.format("%H:%M:%S (UTC%z)").to_string();
            let description = format!("\nDate: {}\nTime: {}", date, time);

            let embeds = [json!({
                "title": title,
                "description": description,
                "color": 0x00ff88
            })];

            let payload = json!({
                "username": bot_name,
                "embeds": embeds
            });

            let _ = client.post(webhook_url).json(&payload).send().await;
        });
    }

    fn send_clock_out_webhook(&self) {
        if self.webhook_url.is_empty() {
            return;
        }

        let client = self.client.clone();
        let webhook_url = self.webhook_url.clone();
        let bot_name = self.bot_name.clone();
        let username = self.username.clone();
        let logs = self.logs.clone();
        let total_time = self.get_verbose_time();

        tokio::spawn(async move {
            let title = format!("{} has clocked out!", username);
            let now = Local::now();
            let date = now.format("%m/%d/%Y").to_string();
            let time = now.format("%H:%M:%S (UTC%z)").to_string();
            let mut description = format!(
                "\nDate: {}\nTime: {}\n\nTotal Logged Time: {}\n\n",
                date, time, total_time
            );

            if logs.is_empty() {
                description.push_str("No logs to display.");
            } else {
                description.push_str("Logs:\n");
                description.push_str(logs.join("\n").as_str());
            };

            let embeds = [json!({
                "title": title,
                "description": description,
                "color": 0x00ff88
            })];

            let payload = json!({
                "username": bot_name,
                "embeds": embeds
            });

            let _ = client.post(webhook_url).json(&payload).send().await;
        });
    }

    fn get_compact_time(&self) -> String {
        let total = self.time;
        let sec = total % 60;
        let min = (total / 60) % 60;
        let hr = (total / 3_600) % 24;
        let days = total / 86_400;

        if days > 0 {
            format!("{}:{:02}:{:02}:{:02}", days, hr, min, sec)
        } else if hr > 0 {
            format!("{:02}:{:02}:{:02}", hr, min, sec)
        } else if min > 0 {
            format!("{:02}:{:02}", min, sec)
        } else {
            format!("{:02}", sec)
        }
    }

    fn get_verbose_time(&self) -> String {
        let total = self.time;
        let sec = total % 60;
        let min = (total / 60) % 60;
        let hr = (total / 3_600) % 24;
        let days = total / 86_400;

        match (days, hr, min) {
            (d, _, _) if d > 0 => {
                format!("{} Days, {} Hours, {} Minutes, {} Seconds", d, hr, min, sec)
            }
            (_, h, _) if h > 0 => {
                format!("{} Hours, {} Minutes, {} Seconds", h, min, sec)
            }
            (_, _, m) if m > 0 => {
                format!("{} Minutes, {} Seconds", m, sec)
            }
            _ => {
                format!("{} Seconds", sec)
            }
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    let username = match env::var("WORKWATCH_USERNAME") {
        Ok(username) => username,
        Err(_) => {
            eprintln!(
                "WorkWatch Warning: WORKWATCH_USERNAME not found! Will default to Anonymous."
            );
            "Anonymous".to_string()
        }
    };

    let webhook_url = match env::var("WORKWATCH_WEBHOOK") {
        Ok(webhook) => webhook,
        Err(_) => {
            eprintln!(
                "WorkWatch Warning: WORKWATCH_WEBHOOK not found! Will not be able to post messages to discord!"
            );
            "".to_string()
        }
    };

    WorkWatcherApp::new(username, webhook_url).run()
}
