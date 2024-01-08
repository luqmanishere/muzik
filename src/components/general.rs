use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
  layout::Rect,
  style::{Color, Style},
  widgets::{Block, Borders, Paragraph, Wrap},
};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
  action::{Action, InputIn, InputOut},
  layouts::{Focus, Scenes},
  mode::Mode,
  tui::Frame,
};

#[derive(Default)]
pub struct TitleBar {}

impl TitleBar {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Component for TitleBar {
  fn draw(&mut self, f: &mut crate::tui::Frame<'_>, area: ratatui::prelude::Rect, _focus: Focus) -> Result<()> {
    let title = Paragraph::new("muzik-tui").alignment(ratatui::layout::Alignment::Left).wrap(Wrap { trim: true });
    f.render_widget(title, area);
    Ok(())
  }

  fn scene(&self) -> crate::layouts::Scenes {
    Scenes::TitleBar
  }

  fn mode(&self) -> crate::mode::Mode {
    Mode::Global
  }
}

#[derive(Default, Debug)]
pub struct InputArea {
  input_name: Option<String>,
  input_buffer: String,
  action_tx: Option<UnboundedSender<Action>>,
  position: usize,
}

impl InputArea {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Component for InputArea {
  fn draw(&mut self, f: &mut Frame<'_>, area: Rect, focus: Focus) -> Result<()> {
    let mut block = Block::default().borders(Borders::ALL);
    if self.is_focused(focus) {
      block = block.border_style(Style { fg: Some(Color::Yellow), ..Default::default() });
      f.set_cursor(area.x + self.position as u16 + 1, area.y + 1)
    }
    if let Some(title) = self.input_name.clone() {
      block = block.title(format!("Input Bar ({})", title));
    } else {
      block = block.title("Input Bar");
    }
    let input = Paragraph::new(self.input_buffer.to_string()).block(block);
    f.render_widget(input, area);
    Ok(())
  }

  fn scene(&self) -> Scenes {
    Scenes::InputBar
  }

  fn mode(&self) -> Mode {
    Mode::Global
  }

  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.action_tx = Some(tx);
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent, focus: Focus) -> Result<Option<Action>> {
    if self.is_focused(focus)
      && key.kind == KeyEventKind::Press
      && (key.modifiers == KeyModifiers::SHIFT || key.modifiers == KeyModifiers::NONE)
    {
      match key.code {
        KeyCode::Char(c) => {
          self.input_buffer.insert(self.position, c);
          self.position += 1;
        },
        KeyCode::Enter => {
          return Ok(Some(Action::InputModeOff(InputOut {
            input_name: self.input_name.clone(),
            buffer: self.input_buffer.clone(),
          })))
        },
        KeyCode::Right => {
          if self.position < self.input_buffer.len() {
            self.position += 1;
          }
        },
        KeyCode::Left => {
          if self.position > 0 {
            self.position -= 1;
          }
        },
        KeyCode::Backspace => {
          // out of bounds is a pain
          if self.position >= 1 {
            // we cannot remove the end of the string
            if self.position == self.input_buffer.len() {
              self.input_buffer.pop();
            } else {
              self.input_buffer.remove(self.position - 1);
            }
            self.position -= 1;
          }
        },
        KeyCode::Esc => return Ok(Some(Action::InputModeOff(InputOut::default()))),
        _ => {},
      }
    }
    Ok(None)
  }

  fn handle_mouse_events(&mut self, mouse: crossterm::event::MouseEvent, focus: Focus) -> Result<Option<Action>> {
    // TODO: maybe mouse input in input bar?
    Ok(None)
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::InputModeOn(InputIn { input_name, initial_value }) => {
        self.input_name = Some(input_name);
        if let Some(initial_value) = initial_value {
          self.input_buffer = initial_value;
          self.position = self.input_buffer.len()
        } else {
          self.input_buffer.clear();
          self.position = 0;
        }
      },
      Action::InputModeOff { .. } => {},
      _ => {},
    }
    Ok(None)
  }
}
