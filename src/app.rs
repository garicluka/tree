use crossterm::{
    cursor,
    event::{KeyCode, KeyEventKind},
    style::{Color, PrintStyledContent, Stylize},
    terminal::{Clear, ClearType},
    QueueableCommand,
};

use std::{
    io::{Stdout, Write},
    path::{Path, PathBuf},
};
use tokio::sync::mpsc;

use crate::{
    tui::{self},
    types::{self, Action, Result},
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
                        self.render(&mut tui.stdout);
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

    fn render(&self, stdout: &mut Stdout) {
        stdout.queue(Clear(ClearType::All)).unwrap();

        Self::render_current_path(stdout, 0, self.current_position, &self.current_path);

        let height = crossterm::terminal::size().unwrap().1;

        for i in 1..height as usize {
            Self::render_child(stdout, i, self.current_position, &self.children);
        }

        stdout.flush().unwrap();
    }

    fn render_current_path(
        stdout: &mut Stdout,
        index: usize,
        current_position: usize,
        current_path: &Path,
    ) {
        let color = if current_position == index {
            Color::DarkGrey
        } else {
            Color::Reset
        };
        Self::line_with_text(
            stdout,
            format!("current path: {:?}", current_path).as_str(),
            index as u16,
            Color::Reset,
            color,
        );
    }

    fn line_with_text(stdout: &mut Stdout, text: &str, position: u16, fg: Color, bg: Color) {
        let (width, _) = crossterm::terminal::size().unwrap();
        let len = text.as_bytes().len();
        for i in 0..width {
            let content = if (i as usize) < len {
                (text.as_bytes()[i as usize] as char).with(fg).on(bg)
            } else {
                ' '.on(bg)
            };

            stdout
                .queue(cursor::MoveTo(i, position))
                .unwrap()
                .queue(PrintStyledContent(content))
                .unwrap();
        }
    }

    fn render_child_inner(
        stdout: &mut Stdout,
        child_index: usize,
        index: usize,
        current_position: usize,
        children: &Vec<PathBuf>,
    ) {
        let color = if current_position == child_index + 1 {
            Color::DarkGrey
        } else {
            Color::Reset
        };
        if child_index >= children.len() {
            return;
        }

        Self::line_with_text(
            stdout,
            format!("current path: {:?}", children[child_index]).as_str(),
            index as u16,
            Color::Reset,
            color,
        );
    }
    fn render_child(
        stdout: &mut Stdout,
        index: usize,
        current_position: usize,
        children: &Vec<PathBuf>,
    ) {
        let height = crossterm::terminal::size().unwrap().1;
        let mid_height = height as usize / 2;
        if current_position < mid_height {
            let child_index = index - 1;
            Self::render_child_inner(stdout, child_index, index, current_position, children);
        } else if current_position >= children.len() - mid_height {
            let child_index = if children.len() < height as usize - 1 {
                index - 1
            } else {
                children.len() + index - height as usize
            };
            Self::render_child_inner(stdout, child_index, index, current_position, children);
        } else {
            let child_index = current_position + index - mid_height;
            Self::render_child_inner(stdout, child_index, index, current_position, children);
        }
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
