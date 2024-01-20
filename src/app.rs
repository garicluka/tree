use crate::{
    tui::{self},
    types::{self, Action, MyError, Result},
    utils::get_current_path,
    virt_terminal::VirtTerminal,
};
use crossterm::{
    event::{KeyCode, KeyEventKind},
    style::Color,
};
use std::{
    io::Stdout,
    path::{Path, PathBuf},
};
use tokio::sync::mpsc;

pub struct App {
    should_quit: bool,
    current_path: PathBuf,
    current_position: usize,
    children: Vec<PathBuf>,
    virt_terminal: VirtTerminal,
}

impl App {
    pub fn new() -> Result<Self> {
        let current_path = get_current_path()?;
        let children = Self::get_all_children(&current_path)?;
        let current_position = 0;
        let should_quit = false;
        let (width, height) = crossterm::terminal::size()?;
        let virt_terminal = VirtTerminal::new(width as usize, height as usize);

        Ok(Self {
            should_quit,
            current_path,
            current_position,
            children,
            virt_terminal,
        })
    }

    pub async fn run(&mut self) -> Result {
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
                                KeyCode::Char('q') => action_tx.send(Action::Quit)?,
                                KeyCode::Char('j') => action_tx.send(Action::MoveDown)?,
                                KeyCode::Char('J') => action_tx.send(Action::MoveDownALOT)?,
                                KeyCode::Char('k') => action_tx.send(Action::MoveUp)?,
                                KeyCode::Char('K') => action_tx.send(Action::MoveUpALOT)?,
                                KeyCode::Char('u') => action_tx.send(Action::Parent)?,
                                KeyCode::Char('d') => action_tx.send(Action::Child)?,
                                _ => {}
                            }
                        }
                    }
                    types::Event::Resize(width, height) => {
                        action_tx.send(Action::Resize(width, height))?
                    }
                    _ => {}
                }
            }

            while let Ok(action) = action_rx.try_recv() {
                match action {
                    Action::Quit => self.should_quit = true,
                    Action::Render => self.render(&mut tui.stdout)?,
                    Action::Resize(width, height) => self.virt_terminal.resize(
                        width as usize,
                        height as usize,
                        &mut tui.stdout,
                    )?,
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

    fn render(&mut self, stdout: &mut Stdout) -> Result {
        self.render_current_path()?;

        for row in 1..self.virt_terminal.height {
            self.render_child(row)?;
        }

        self.virt_terminal.flush(stdout)?;

        Ok(())
    }

    fn render_current_path(&mut self) -> Result {
        let color = if self.current_position == 0 {
            Color::DarkGrey
        } else {
            Color::Reset
        };

        let current_path = if let Some(path) = self.current_path.to_str() {
            path
        } else {
            return Err(Box::from(MyError::new(
                "cannot translate current_path to &str",
            )));
        };

        self.line_with_text(
            format!("current path: {}", current_path).as_str(),
            0,
            Color::Reset,
            color,
        )?;

        Ok(())
    }

    fn render_child(&mut self, row: usize) -> Result {
        let height = self.virt_terminal.height;
        let mid_height = height / 2;

        if self.current_position < mid_height {
            let child_index = row - 1;
            self.render_child_inner(child_index, row)?;
        } else if self.current_position >= self.children.len() - mid_height {
            let child_index = if self.children.len() < height - 1 {
                row - 1
            } else {
                self.children.len() + row - height
            };
            self.render_child_inner(child_index, row)?;
        } else {
            let child_index = self.current_position + row - mid_height;
            self.render_child_inner(child_index, row)?;
        }

        Ok(())
    }

    fn render_child_inner(&mut self, child_index: usize, row: usize) -> Result {
        if child_index >= self.children.len() {
            return Ok(());
        }

        let color = if self.current_position == child_index + 1 {
            Color::DarkGrey
        } else {
            Color::Reset
        };

        let temp_child_path = self.children[child_index].clone();
        let child_path = if let Some(path) = temp_child_path.to_str() {
            path
        } else {
            return Err(Box::from(MyError::new(
                "cannot translate child_path to &str",
            )));
        };

        self.line_with_text(child_path, row, Color::Reset, color)?;

        Ok(())
    }

    fn line_with_text(&mut self, text: &str, row: usize, fg: Color, bg: Color) -> Result {
        let len = text.as_bytes().len();

        for col in 0..self.virt_terminal.width {
            let content = if col < len {
                text.as_bytes()[col] as char
            } else {
                ' '
            };
            self.virt_terminal.change_cell(row, col, content, fg, bg);
        }

        Ok(())
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
