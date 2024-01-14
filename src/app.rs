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
                                KeyCode::Char('J') => {
                                    action_tx.send(Action::MoveDownALOT)?;
                                }
                                KeyCode::Char('k') => {
                                    action_tx.send(Action::MoveUp)?;
                                }
                                KeyCode::Char('K') => {
                                    action_tx.send(Action::MoveUpALOT)?;
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
                    Action::MoveUpALOT => {
                        if self.current_position > 9 {
                            self.current_position -= 10;
                        } else {
                            self.current_position = 0;
                        }
                    }
                    Action::MoveDownALOT => {
                        if self.current_position + 9 < self.children.len() {
                            self.current_position += 10;
                        } else {
                            self.current_position = self.children.len();
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
        fn render_current_path(
            f: &mut Frame,
            index: usize,
            area: &Rect,
            current_position: usize,
            current_path: &Path,
        ) {
            let color = if current_position == index {
                Color::DarkGray
            } else {
                Color::Reset
            };
            f.render_widget(
                Paragraph::new(format!("current path: {:?}", current_path)).bg(color),
                *area,
            );
        }

        fn render_child_inner(
            f: &mut Frame,
            child_index: usize,
            area: &Rect,
            current_position: usize,
            children: &Vec<PathBuf>,
        ) {
            let color = if current_position == child_index + 1 {
                Color::DarkGray
            } else {
                Color::Reset
            };
            if child_index >= children.len() {
                return;
            }
            f.render_widget(
                Paragraph::new(format!("{:?}", children[child_index])).bg(color),
                *area,
            );
        }

        fn render_child(
            f: &mut Frame,
            index: usize,
            area: &Rect,
            current_position: usize,
            children: &Vec<PathBuf>,
        ) {
            let mid_height = f.size().height as usize / 2;

            if current_position < mid_height {
                let child_index = index - 1;
                render_child_inner(f, child_index, area, current_position, children);
            } else if current_position >= children.len() - mid_height {
                let child_index = if children.len() < f.size().height as usize - 1 {
                    index - 1
                } else {
                    children.len() + index - f.size().height as usize
                };
                render_child_inner(f, child_index, area, current_position, children);
            } else {
                let child_index = current_position + index - mid_height;
                render_child_inner(f, child_index, area, current_position, children);
            }
        }

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
                    render_current_path(f, index, area, self.current_position, &self.current_path);
                } else {
                    render_child(f, index, area, self.current_position, &self.children);
                }
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
