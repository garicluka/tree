use crossterm::event::{KeyEvent, MouseEvent};

pub type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

pub enum Action {
    Render,
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
