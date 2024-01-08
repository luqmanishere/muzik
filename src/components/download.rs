//! This module contains components related to the download mode of the program

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
  layout::{Constraint, Layout},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tracing::{debug, info, trace, warn};
use youtube_dl::{SearchOptions, SingleVideo, YoutubeDl, YoutubeDlOutput};

use super::Component;
use crate::{
  action::{Action, InputIn, InputOut},
  layouts::{Focus, Scenes},
  mode::Mode,
};

#[derive(Default)]
pub struct SearchBar {
  search_query: String,
  action_tx: Option<UnboundedSender<Action>>,
  current_mode: Mode,
}

impl SearchBar {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Component for SearchBar {
  fn draw(&mut self, f: &mut crate::tui::Frame<'_>, area: ratatui::prelude::Rect, focus: Focus) -> Result<()> {
    let text = if self.search_query.is_empty() {
      "Press <s> to begin search".to_string()
    } else {
      format!("Searching for {}...", self.search_query)
    };

    let block = Block::default().borders(Borders::ALL).title("Search Query");

    let para = Paragraph::new(text).block(block);
    f.render_widget(para, area);
    Ok(())
  }

  fn scene(&self) -> crate::layouts::Scenes {
    Scenes::Download(crate::layouts::DownloadLayouts::SearchBar)
  }

  fn mode(&self) -> crate::mode::Mode {
    crate::mode::Mode::Download
  }

  fn register_action_handler(&mut self, tx: tokio::sync::mpsc::UnboundedSender<crate::action::Action>) -> Result<()> {
    self.action_tx = Some(tx);
    Ok(())
  }

  fn handle_key_events(
    &mut self,
    key: crossterm::event::KeyEvent,
    focus: Focus,
  ) -> Result<Option<crate::action::Action>> {
    if focus.mode == self.mode() && key.modifiers == KeyModifiers::NONE && key.code == KeyCode::Char('s') {
      return Ok(Some(Action::InputModeOn(InputIn { input_name: "youtube_search".to_string(), initial_value: None })));
    }
    Ok(None)
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      // woah that collapsible matching clippy hint was cool af
      Action::InputModeOff(InputOut { input_name: Some(input_name), buffer }) => {
        if input_name == *"youtube_search" {
          self.search_query = buffer;
          // we will not be the component that sends the search request
        }
      },
      _ => {},
    }
    Ok(None)
  }
}

#[derive(Default, Debug)]
pub struct SearchResult {
  search_query: String,
  search_rx: Option<oneshot::Receiver<Result<YoutubeDlOutput, youtube_dl::Error>>>,
  search_result_videos: Option<Vec<SingleVideo>>,
  search_result_list_state: ListState,
}

impl SearchResult {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn list_next(&mut self) {
    if let Some(videos) = &self.search_result_videos {
      if let Some(index) = self.search_result_list_state.selected() {
        if index >= videos.len() - 1 {
          self.search_result_list_state.select(Some(0));
        } else {
          self.search_result_list_state.select(Some(index + 1));
        }
        return;
      }
    }
    self.search_result_list_state.select(Some(0));
  }

  pub fn previous_list(&mut self) {
    if let Some(videos) = &self.search_result_videos {
      if let Some(index) = self.search_result_list_state.selected() {
        if index == 0 {
          self.search_result_list_state.select(Some(videos.len() - 1));
        } else {
          self.search_result_list_state.select(Some(index - 1))
        }
        return;
      }
    }
    self.search_result_list_state.select(Some(0));
  }

  pub fn unselect_list(&mut self) {
    self.search_result_list_state.select(None);
  }

  fn get_current_selected_list_youtube_video(&self) -> Option<YoutubeVideo> {
    if let Some(index) = self.search_result_list_state.selected() {
      if let Some(videos) = &self.search_result_videos {
        match videos.get(index) {
          Some(video) => return Some(video.to_owned().into()),
          None => return None,
        }
      }
    }
    None
  }
}

impl Component for SearchResult {
  fn draw(&mut self, f: &mut crate::tui::Frame<'_>, area: ratatui::prelude::Rect, focus: Focus) -> Result<()> {
    let divider = Block::default().borders(Borders::RIGHT);
    if let Some(videos) = &self.search_result_videos {
      let list_item: Vec<_> =
        videos.iter().map(|e| ListItem::new(e.title.clone().unwrap_or("Unknown".to_string()))).collect();
      let list = List::new(list_item).highlight_symbol(">>").block(divider);
      f.render_stateful_widget(list, area, &mut self.search_result_list_state);
    } else {
      f.render_widget(Paragraph::new("Nothing searched yet"), area);
    }
    Ok(())
  }

  fn scene(&self) -> Scenes {
    Scenes::Download(crate::layouts::DownloadLayouts::SearchResult)
  }

  fn mode(&self) -> Mode {
    Mode::Download
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::Tick => {
        if let Some(search_rx) = &mut self.search_rx {
          match search_rx.try_recv() {
            Ok(result) => {
              info!("youtube_search oneshot returned");
              match result {
                Ok(result) => {
                  let videos = result.into_playlist().expect("playlist");
                  let videos = videos.entries.expect("vec of videos");
                  self.search_result_videos = Some(videos);
                },
                Err(e) => return Ok(Some(Action::Error(format!("youtube search failed: {e}")))),
              }
            },
            Err(oneshot::error::TryRecvError::Empty) => {
              trace!("youtube_search oneshot channel is empty");
            },
            Err(oneshot::error::TryRecvError::Closed) => {
              self.search_rx = None;
              warn!("youtube search oneshot channel closed");
            },
          }
        }
      },
      Action::InputModeOff(InputOut { input_name, buffer }) => {
        if let Some(input_name) = input_name {
          if input_name == *"youtube_search" {
            self.search_query = buffer;
            // build the search request
            let search_query = self.search_query.clone();
            let (ys_tx, ys_rx) = tokio::sync::oneshot::channel();
            self.search_rx = Some(ys_rx);
            tokio::spawn(async move {
              let youtube_search =
                YoutubeDl::search_for(&SearchOptions::youtube(search_query).with_count(15)).run_async().await;
              ys_tx.send(youtube_search).unwrap();
            });
            debug!("started youtube search task");
          };
        }
      },
      _ => {},
    }
    Ok(None)
  }

  fn handle_key_events(&mut self, key: crossterm::event::KeyEvent, focus: Focus) -> Result<Option<Action>> {
    if self.is_focused(focus) && key.modifiers == KeyModifiers::NONE {
      match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
          self.list_next();
          return Ok(Some(Action::DownloadShowSearchDetails(self.get_current_selected_list_youtube_video())));
        },
        KeyCode::Char('k') | KeyCode::Up => {
          self.previous_list();
          return Ok(Some(Action::DownloadShowSearchDetails(self.get_current_selected_list_youtube_video())));
        },
        KeyCode::Esc => {
          if self.search_result_list_state.selected().is_some() {
            self.unselect_list();
          } else {
            return Ok(Some(Action::FocusBack));
          }
          return Ok(Some(Action::DownloadShowSearchDetails(None)));
        },
        _ => {},
      }
    }
    Ok(None)
  }
}

/// Struct showing the details of the selected search result
#[derive(Default, Debug)]
pub struct SearchResultDetails {
  selected_search_result: Option<YoutubeVideo>,
}

impl SearchResultDetails {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Component for SearchResultDetails {
  fn draw(&mut self, f: &mut crate::tui::Frame<'_>, area: ratatui::prelude::Rect, _focus: Focus) -> Result<()> {
    if let Some(video) = &self.selected_search_result {
      let layout =
        Layout::new(ratatui::layout::Direction::Vertical, [Constraint::Length(1), Constraint::Min(1)]).split(area);

      let desc = Paragraph::new("Details").alignment(ratatui::layout::Alignment::Center);
      f.render_widget(desc, layout[0]);

      let id = ListItem::new(format!("Id: {}", video.id.clone()));
      let title = ListItem::new(format!("Title: {}", video.title.clone().unwrap_or("Unknown".to_string())));
      let channel = ListItem::new(format!("Channel: {}", video.channel.clone().unwrap_or("Unknown".to_string())));
      let artist = ListItem::new(format!("Artist: {}", video.artist.clone().unwrap_or("Unknown".to_string())));
      let album = ListItem::new(format!("Album: {}", video.album.clone().unwrap_or("Unknown".to_string())));
      let list = List::new([id, title, channel, artist, album]);
      f.render_widget(list, layout[1]);
    } else {
      let placeholder = Paragraph::new("Nothing to display yet");
      f.render_widget(placeholder, area);
    }
    Ok(())
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::DownloadShowSearchDetails(youtube_details) => {
        self.selected_search_result = youtube_details;
        //
      },
      _ => {},
    }
    Ok(None)
  }

  fn scene(&self) -> Scenes {
    Scenes::Download(crate::layouts::DownloadLayouts::SearchResultDetails)
  }

  fn mode(&self) -> Mode {
    Mode::Download
  }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct YoutubeVideo {
  id: String,
  title: Option<String>,
  channel: Option<String>,
  album: Option<String>,
  artist: Option<String>,
  genre: Option<String>,
}

impl From<SingleVideo> for YoutubeVideo {
  fn from(value: SingleVideo) -> Self {
    Self {
      id: value.id,
      title: value.title,
      channel: value.channel,
      album: value.album,
      artist: value.artist,
      genre: value.genre,
    }
  }
}
