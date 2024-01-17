use std::{error::Error, fmt::Display};

use crossterm::event::{KeyEvent, MouseEvent};

pub type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct MyError {
    msg: String,
}

impl MyError {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)?;
        Ok(())
    }
}

impl Error for MyError {}

pub enum Action {
    Render,
    Resize(u16, u16),
    Quit,
    Parent,
    Child,
    MoveUp,
    MoveUpALOT,
    MoveDown,
    MoveDownALOT,
}

pub enum Event {
    Init,
    Error,
    Tick,
    Render,
    FocusLost,
    FocusGained,
    Paste(String),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Key(KeyEvent),
}
