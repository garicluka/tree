use std::path::{Path, PathBuf};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Stylize},
    widgets::Paragraph,
    Frame,
};
use tokio::sync::mpsc;

use crate::{
    tui::{self},
    types::{self, Action},
    utils::get_current_path,
};

pub struct App {
    should_quit: bool,
    current_path: PathBuf,
    current_position: usize,
    children: Vec<PathBuf>,
}

impl App {
    pub fn new() -> Result<Self> {
        let current_path = get_current_path()?;
        let children = Self::get_all_children(&current_path)?;
        let current_position = 0;
        let should_quit = false;

        Ok(Self {
            should_quit,
            current_path,
            current_position,
            children,
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
                    Action::Quit => self.should_quit = true,
                    Action::Render => {
                        tui.terminal.draw(|f| self.render(f))?;
                    }
                    Action::Parent => {
                        if let Some(parent) = self.current_path.parent() {
                            self.current_path = parent.to_path_buf();
                            self.current_position = 0;
                            self.children = Self::get_all_children(&self.current_path)?;
                        }
                    }
                    Action::Child => {
                        if self.current_position != 0 {
                            let child = self.children[self.current_position - 1].clone();
                            self.current_path = child;
                            self.current_position = 0;
                            self.children = Self::get_all_children(&self.current_path)?;
                        }
                    }
                    Action::MoveUp => {
                        if self.current_position > 0 {
                            self.current_position -= 1;
                        }
                    }
                    Action::MoveDown => {
                        if self.current_position < self.children.len() {
                            self.current_position += 1;
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

    fn render(&mut self, f: &mut Frame) {
        let mut constraints = Vec::new();
        constraints.push(Constraint::Length(1));

        constraints.extend_from_slice(&vec![Constraint::Length(1); self.children.len()]);

        constraints.push(Constraint::Percentage(100));

        let layout = Layout::new(Direction::Vertical, &constraints).split(f.size());
        layout.iter().enumerate().for_each(|(index, area)| {
            if index == layout.len() - 1 {
                return;
            }
            let color = if self.current_position == index {
                Color::Gray
            } else {
                Color::Reset
            };
            if index == 0 {
                f.render_widget(
                    Paragraph::new(format!("current path: {:?}", self.current_path)).bg(color),
                    *area,
                );
                return;
            }
            f.render_widget(
                Paragraph::new(format!("{:?}", self.children[index - 1])).bg(color),
                *area,
            );
        });
    }

    fn get_all_children(path: &Path) -> Result<Vec<PathBuf>> {
        let mut all_children = vec![];

        if path.is_dir() {
            for entry in path.read_dir()? {
                all_children.push(entry?.path());
            }
        }

        Ok(all_children)
    }
}
