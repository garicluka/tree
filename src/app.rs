use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    widgets::Paragraph,
};
use tokio::sync::mpsc;

use crate::{
    tui,
    types::{self, Action},
    utils::get_current_path,
};

pub struct App {
    should_quit: bool,
    current_path: PathBuf,
    current_position: u16,
}

impl App {
    pub fn new() -> Result<Self> {
        let current_path = get_current_path()?;
        Ok(Self {
            should_quit: false,
            current_path,
            current_position: 0,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();
        let mut tui = tui::Tui::new()?;

        tui.start()?;

        loop {
            if let Some(event) = tui.next_event().await {
                match event {
                    types::Event::Render => action_tx.send(Action::Render)?,
                    types::Event::Key(key) => {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Char('q') => {
                                    action_tx.send(Action::Quit)?;
                                }
                                KeyCode::Char('j') => {
                                    action_tx.send(Action::MoveDown)?;
                                }
                                KeyCode::Char('k') => {
                                    action_tx.send(Action::MoveUp)?;
                                }
                                KeyCode::Char('u') => {
                                    action_tx.send(Action::Parent)?;
                                }
                                KeyCode::Char('d') => {
                                    action_tx.send(Action::Child)?;
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }

            while let Ok(action) = action_rx.try_recv() {
                match action {
                    Action::Quit => {
                        self.should_quit = true;
                    }
                    Action::Render => {
                        tui.terminal.draw(|f| {
                            let mut children = vec![];

                            if self.current_path.is_dir() {
                                let read_dir = if let Ok(dir) = self.current_path.read_dir() {
                                    dir
                                } else {
                                    f.render_widget(
                                        Paragraph::new("Cannot read directory!")
                                            .bg(Color::Red)
                                            .fg(Color::White),
                                        Rect::new(0, 0, f.size().width, 1),
                                    );
                                    return;
                                };
                                for entry in read_dir.flatten() {
                                    children.push(entry.path());
                                }
                            }

                            let mut constraints = vec![];
                            constraints.push(Constraint::Length(1));
                            constraints
                                .extend_from_slice(&vec![Constraint::Length(1); children.len()]);
                            constraints.push(Constraint::Percentage(100));

                            let layout =
                                Layout::new(Direction::Vertical, constraints).split(f.size());

                            let color = if self.current_position == 0 {
                                Color::Gray
                            } else {
                                Color::Reset
                            };

                            f.render_widget(
                                Paragraph::new(format!("current path: {:?}", self.current_path))
                                    .bg(color),
                                layout[0],
                            );

                            for (index, child) in children.iter().enumerate() {
                                let color = if self.current_position == index as u16 + 1 {
                                    Color::Gray
                                } else {
                                    Color::Reset
                                };

                                f.render_widget(
                                    Paragraph::new(format!("{:?}", child)).bg(color),
                                    layout[index + 1],
                                );
                            }
                        })?;
                    }
                    Action::Parent => {
                        if let Some(parent) = self.current_path.parent() {
                            self.current_path = parent.to_path_buf();
                            self.current_position = 0;
                        }
                    }
                    Action::Child => {
                        if self.current_position != 0 {
                            let mut children = vec![];
                            let read_dir = self.current_path.read_dir()?;
                            for entry in read_dir.flatten() {
                                children.push(entry.path());
                            }
                            let child = children[self.current_position as usize - 1].clone();
                            self.current_path = child;
                            self.current_position = 0;
                        }
                    }
                    Action::MoveUp => {
                        if self.current_position > 0 {
                            self.current_position -= 1;
                        }
                    }
                    Action::MoveDown => {
                        if self.current_path.is_dir() {
                            let mut children = vec![];
                            let read_dir = self.current_path.read_dir()?;
                            for entry in read_dir.flatten() {
                                children.push(entry.path());
                            }
                            if self.current_position < children.len() as u16 {
                                self.current_position += 1;
                            }
                        }
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }
        tui.stop()?;

        Ok(())
    }
}
