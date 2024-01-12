// use anyhow::Result;
// use ratatui::{layout::Rect, widgets::Paragraph, Frame};
// use std::path::PathBuf;
// use tokio::sync::mpsc::UnboundedSender;
//
// use crate::{
//     types::{Action, Event},
//     utils::get_current_path,
// };
//
// #[derive(Default)]
// pub struct Home {
//     path: PathBuf,
//     action_tx: Option<UnboundedSender<Action>>,
// }
//
// impl Home {
//     pub fn new() -> Result<Self> {
//         let path = get_current_path()?;
//         Ok(Self {
//             path,
//             action_tx: None,
//         })
//     }
//
//     pub fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {
//         self.action_tx = Some(tx);
//     }
//
//     pub fn update(&self, _action: Action) {}
//
//     pub fn handle_events(&self, _event: &Event) {}
//
//     pub fn view(&self, area: Rect, f: &mut Frame<'_>) {
//         f.render_widget(Paragraph::new(format!("{:?}", self.path)), area);
//     }
// }
