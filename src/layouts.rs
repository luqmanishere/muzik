use std::collections::HashMap;

use color_eyre::eyre::{eyre, OptionExt, Result};
use ratatui::layout::{Constraint, Layout, Rect};
use strum::Display;
use tracing::{debug, warn};

use crate::{components::Component, mode::Mode};

/// Enum of screens or individual elements
#[derive(Hash, Debug, Eq, PartialEq, Display, Clone)]
pub enum Scenes {
  Home(HomeLayouts),
  Download(DownloadLayouts),
  Manager(ManagerLayouts),
  InputBar,
  TitleBar,
}

impl Default for Scenes {
  fn default() -> Self {
    Scenes::Home(HomeLayouts::default())
  }
}

#[derive(Default, Hash, Debug, Eq, PartialEq, Display, Clone)]
pub enum HomeLayouts {
  #[default]
  Intro,
}

#[derive(Default, Hash, Debug, Eq, PartialEq, Display, Clone)]
pub enum DownloadLayouts {
  #[default]
  SearchBar,
  SearchResult,
  SearchResultDetails,
}

#[derive(Default, Hash, Debug, Eq, PartialEq, Display, Clone)]
pub enum ManagerLayouts {
  #[default]
  SongList,
}

#[derive(Default, Debug)]
pub enum Orientation {
  #[default]
  Landscape,
  Portrait,
}

/// Manages all predefined layouts in the application
/// Components should request a layout from the manager
/// If no other componenet is rendering with the layout then the layout should be returned
///
/// If there is a conflict, log the error and provide the layout anyways
#[derive(Default)]
pub struct LayoutManager {
  layout_store: HashMap<Scenes, Rect>,
  screen: Rect,
  orientation: Orientation,
}

impl LayoutManager {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn init(&mut self, screen: Rect) -> Result<()> {
    self.update(screen)?;
    Ok(())
  }

  pub fn get_component_layout(&self, layout_key: Scenes) -> Result<Rect> {
    return self.layout_store.get(&layout_key).ok_or_eyre("Layout key {layout_key} does not exists").copied();
  }

  /// On terminal resize, update the screen sizing then trigger a layout rebuild
  pub fn update(&mut self, screen: Rect) -> Result<()> {
    self.screen = screen;
    self.build_layouts()?;
    Ok(())
  }

  fn build_download_layout(&mut self, area: Rect) -> Result<()> {
    let vertical_layout = Layout::default()
      .direction(ratatui::layout::Direction::Vertical)
      .constraints([Constraint::Length(3), Constraint::Min(1)])
      .split(area);

    let horizontal_layout = Layout::new(ratatui::layout::Direction::Horizontal, Constraint::from_percentages([50, 50]))
      .split(vertical_layout[1]);

    self.layout_store.insert(Scenes::Download(DownloadLayouts::SearchBar), vertical_layout[0]);
    self.layout_store.insert(Scenes::Download(DownloadLayouts::SearchResult), horizontal_layout[0]);
    self.layout_store.insert(Scenes::Download(DownloadLayouts::SearchResultDetails), horizontal_layout[1]);
    Ok(())
  }

  /// Build layouts based on screen size. Might be expensive
  fn build_layouts(&mut self) -> Result<()> {
    let layout = Layout::default()
      .direction(ratatui::layout::Direction::Vertical)
      .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(3)])
      .split(self.screen);
    // Default elements present in every screen
    self.layout_store.insert(Scenes::TitleBar, layout[0]);
    self.layout_store.insert(Scenes::InputBar, layout[2]);

    let main_render_area = layout[1];

    // Screen: Home
    self.layout_store.insert(Scenes::Home(HomeLayouts::Intro), main_render_area);

    self.build_download_layout(main_render_area)?;
    Ok(())
  }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Focus {
  pub mode: Mode,
  pub scene: Scenes,
}
