use std::{io, time::Duration};

use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::Rect,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

#[derive(Default)]
pub struct WorkWatcherApp {
    time: u32,
    exit: bool,
}

impl WorkWatcherApp {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_secs(1))? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind == KeyEventKind::Press {
                        if let KeyCode::Char('q') = key_event.code {
                            self.exit()
                        }
                    }
                }
            } else {
                self.time = self.time.saturating_add(1);
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
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

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &WorkWatcherApp {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(Line::from(format!(
            "Logged Time: {}",
            self.get_compact_time()
        )))
        .block(Block::bordered().title(Line::from(" Work Watcher ")))
        .render(area, buf);
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut work_watcher_app = WorkWatcherApp::default();
    let result = work_watcher_app.run(&mut terminal);
    ratatui::restore();
    println!("Total Logged Time: {}", work_watcher_app.get_verbose_time());
    result
}
