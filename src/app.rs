use anyhow::Result;
use crossterm::event::{KeyCode, KeyEventKind};
use tokio::sync::mpsc;

use crate::{
    home::Home,
    tui,
    types::{self, Action},
};

pub struct App {
    should_quit: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self { should_quit: false })
    }
    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();
        let mut tui = tui::Tui::new()?;
        tui.start()?;
        let mut home = Home::new()?;
        home.register_action_handler(action_tx.clone());

        loop {
            if let Some(event) = tui.next_event().await {
                match event {
                    types::Event::Render => action_tx.send(Action::Render)?,
                    types::Event::Key(key) => {
                        if key.kind == KeyEventKind::Press {
                            if let KeyCode::Char('q') = key.code {
                                action_tx.send(Action::Quit)?;
                            }
                        }
                    }
                    _ => {}
                }
                home.handle_events(&event);
            }

            while let Ok(action) = action_rx.try_recv() {
                match action {
                    Action::Quit => {
                        self.should_quit = true;
                    }
                    Action::Render => {
                        tui.terminal.draw(|f| {
                            home.view(f.size(), f);
                        })?;
                    }
                }
                home.update(action);
            }

            if self.should_quit {
                break;
            }
        }
        tui.stop()?;

        Ok(())
    }
}
