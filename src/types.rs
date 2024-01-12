use crossterm::event::{KeyEvent, MouseEvent};

pub enum Action {
    Render,
    Quit,
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
