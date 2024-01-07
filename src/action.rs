use std::{fmt, string::ToString};

use serde::{
  de::{self, Deserializer, Visitor},
  Deserialize, Serialize,
};
use strum::Display;
use youtube_dl::SingleVideo;

use crate::{components::download::YoutubeVideo, layouts::Focus, mode::Mode};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum Action {
  /// A tick event from the event loop
  Tick,
  /// Tells the run loop to render
  Render,
  /// Refresh the UI layout upon refresh
  Resize(u16, u16),
  Suspend,
  Resume,
  /// Cleanly exit the program
  Quit,
  Refresh,
  Error(String),
  Help,
  /// Switch to the given scene
  FocusSwitch(#[serde(skip)] Focus),
  FocusBack,

  /// Toggles Input Mode on
  ///
  /// This causes the run loop to avoid invoking any actions during input
  ///
  /// # Arguments
  ///
  /// * String: a unique name for the input so as to not clash with other components
  /// * String (optional): Text as initial value
  // InputModeOn(#[serde(skip)] (String, Option<String>)),
  InputModeOn(#[serde(skip)] InputIn),
  /// Toggles input mode off
  ///
  /// The run loop will resume actions parsing
  ///
  /// # Arguments
  ///
  /// * String (Optional): A string will indicate successful input, otherwise a cancellation or
  /// other errors
  /// * String: the buffer contents upon exit from Input Mode
  // InputModeOff(#[serde(skip)] (Option<String>, String)),
  InputModeOff(#[serde(skip)] InputOut),

  DownloadSearchYoutube,
  DownloadShowSearchDetails(#[serde(skip)] Option<YoutubeVideo>),
  DownloadSearchToDetails,
}

#[derive(Clone, Debug, Eq, Default, PartialEq)]
pub struct InputIn {
  pub input_name: String,
  pub initial_value: Option<String>,
}

#[derive(Clone, Debug, Eq, Default, PartialEq)]
pub struct InputOut {
  pub input_name: Option<String>,
  pub buffer: String,
}
