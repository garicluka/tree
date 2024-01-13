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
    current_position: u16,
    all_children: Vec<PathBuf>,
}

impl App {
    pub fn new() -> Result<Self> {
        let current_path = get_current_path()?;
        let all_children = Self::get_all_children(&current_path)?;
        let should_quit = false;
        let current_position = 0;

        Ok(Self {
            should_quit,
            current_path,
            current_position,
            all_children,
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
                            self.all_children = Self::get_all_children(&self.current_path)?;
                        }
                    }
                    Action::Child => {
                        if self.current_position != 0 {
                            let child =
                                self.all_children[self.current_position as usize - 1].clone();
                            self.current_path = child;
                            self.current_position = 0;
                            self.all_children = Self::get_all_children(&self.current_path)?;
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
                            if self.current_position < self.all_children.len() as u16 {
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

    fn render(&mut self, f: &mut Frame) {
        let mut constraints = Vec::new();
        constraints.push(Constraint::Length(1));
        constraints.extend_from_slice(&vec![Constraint::Length(1); self.all_children.len()]);
        constraints.push(Constraint::Percentage(100));

        let layout = Layout::new(Direction::Vertical, &constraints).split(f.size());
        layout.iter().enumerate().for_each(|(index, area)| {
            if index == layout.len() - 1 {
                return;
            }
            let color = if self.current_position == index as u16 {
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
                Paragraph::new(format!("{:?}", self.all_children[index - 1])).bg(color),
                *area,
            );
        });
    }

    fn get_all_children(path: &Path) -> Result<Vec<PathBuf>> {
        let mut children = vec![];

        if path.is_dir() {
            for entry in path.read_dir()? {
                children.push(entry?.path());
            }
        }

        Ok(children)
    }
}
