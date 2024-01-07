use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use super::{Component, Frame};
use crate::{
  action::Action,
  config::{Config, KeyBindings},
  layouts::{Focus, HomeLayouts, Scenes},
  mode::Mode,
};

#[derive(Default)]
pub struct Intro {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
}

impl Intro {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Component for Intro {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::Tick => {},
      _ => {},
    }
    Ok(None)
  }

  fn handle_key_events(&mut self, key: KeyEvent, focus: Focus) -> Result<Option<Action>> {
    if focus.mode == self.mode() && focus.scene == self.scene() {
      if let KeyCode::Enter = key.code {
        return Ok(Some(Action::FocusSwitch(Focus {
          mode: Mode::Download,
          // move to search result because its the first interactable
          scene: Scenes::Download(crate::layouts::DownloadLayouts::SearchResult),
        })));
      }
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect, focus: Focus) -> Result<()> {
    let intro_text = Paragraph::new("Welcome to muzik-tui!\nPress <Enter> to start download.\nPress <l> to go to the management list.\nPress <q> to exit at anytime")
      .alignment(Alignment::Center)
      .block(
        Block::default().borders(Borders::ALL).padding(Padding { top: (area.height / 2) - 2, ..Default::default() }),
      );
    f.render_widget(intro_text, area);
    Ok(())
  }

  fn scene(&self) -> Scenes {
    Scenes::Home(HomeLayouts::Intro)
  }

  fn mode(&self) -> Mode {
    Mode::Home
  }
}
