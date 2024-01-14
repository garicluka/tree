use std::path::{Path, PathBuf};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
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
        let constraints: Vec<Constraint> = [
            vec![Constraint::Length(1)],
            vec![Constraint::Length(1); f.size().height as usize - 1],
        ]
        .concat();

        Layout::new(Direction::Vertical, constraints)
            .split(f.size())
            .iter()
            .enumerate()
            .for_each(|(index, area)| {
                if index == 0 {
                    self.render_current_path(f, index, area);
                } else {
                    self.render_child(f, index, area);
                }
            });
    }

    fn render_current_path(&self, f: &mut Frame, index: usize, area: &Rect) {
        let color = if self.current_position == index {
            Color::Gray
        } else {
            Color::Reset
        };
        f.render_widget(
            Paragraph::new(format!("current path: {:?}", self.current_path)).bg(color),
            *area,
        );
    }

    fn render_child(&self, f: &mut Frame, index: usize, area: &Rect) {
        let mid_height = f.size().height as usize / 2;

        if self.current_position < mid_height {
            let child_index = index - 1;
            self.render_child_inner(f, child_index, area);
        } else if self.current_position >= self.children.len() - mid_height {
            let child_index = if self.children.len() < f.size().height as usize - 1 {
                index - 1
            } else {
                self.children.len() + index - f.size().height as usize
            };
            self.render_child_inner(f, child_index, area);
        } else {
            let child_index = self.current_position + index - mid_height;
            self.render_child_inner(f, child_index, area);
        }
    }

    fn render_child_inner(&self, f: &mut Frame, child_index: usize, area: &Rect) {
        let color = if self.current_position == child_index + 1 {
            Color::Gray
        } else {
            Color::Reset
        };
        if child_index >= self.children.len() {
            return;
        }
        f.render_widget(
            Paragraph::new(format!("{:?}", self.children[child_index])).bg(color),
            *area,
        );
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
