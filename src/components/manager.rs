use color_eyre::eyre::{eyre, Result};
use ratatui::prelude::*;

use super::Component;
use crate::{
  config::Config,
  layouts::{Focus, ManagerLayouts, Scenes},
  mode::Mode,
};

#[derive(Default, Clone, Debug)]
pub enum DisplayMode {
  #[default]
  Local,
  Database,
  All,
}

#[derive(Default)]
pub struct SongList {
  display_mode: DisplayMode,
  config: Option<Config>,
}

impl SongList {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Component for SongList {
  fn draw(&mut self, f: &mut crate::tui::Frame<'_>, area: Rect, focus: Focus) -> color_eyre::eyre::Result<()> {
    Ok(())
  }

  fn scene(&self) -> Scenes {
    Scenes::Manager(ManagerLayouts::SongList)
  }

  fn mode(&self) -> Mode {
    Mode::Manager
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = Some(config);
    Ok(())
  }
}
