use std::sync::Arc;

use color_eyre::eyre::{ContextCompat, Result};
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::error;

use crate::{
  action::Action,
  components::{
    download,
    fps::FpsCounter,
    general::{InputArea, TitleBar},
    home::Intro,
    manager, Component,
  },
  config::Config,
  database::Database,
  layouts::{Focus, HomeLayouts, LayoutManager, Scenes},
  mode::Mode,
  tui,
};

pub struct App {
  /// App config
  pub config: Config,
  /// polling rate
  pub tick_rate: f64,
  /// rendering frame rate
  pub frame_rate: f64,
  /// components to be rendered
  pub components: Vec<Box<dyn Component>>,
  pub should_quit: bool,
  pub should_suspend: bool,
  /// layout manager
  pub layout_manager: LayoutManager,
  pub last_tick_key_events: Vec<KeyEvent>,
  pub focus_buffer: Vec<Focus>,

  pub database: Database,
}

impl App {
  /// create new instance of app
  pub async fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
    let home = Intro::new();
    let fps = FpsCounter::default();
    let config = Config::new()?;
    let mode = Mode::Home;
    let first_focus = Focus { mode, scene: Scenes::Home(HomeLayouts::Intro) };
    let layout_manager = LayoutManager::new();
    let components: Vec<Box<(dyn Component + 'static)>> = vec![
      Box::new(home),
      Box::new(fps),
      Box::new(TitleBar::new()),
      Box::new(InputArea::new()),
      Box::new(download::SearchBar::new()),
      Box::new(download::SearchResult::new()),
      Box::new(download::SearchResultDetails::new()),
      Box::new(manager::SongList::new()),
    ];

    let database = Database::new(config.clone()).await?;
    Ok(Self {
      tick_rate,
      frame_rate,
      components,
      should_quit: false,
      should_suspend: false,
      config,
      layout_manager,
      last_tick_key_events: Vec::new(),
      focus_buffer: vec![first_focus],
      database,
    })
  }

  // main app running function
  pub async fn run(&mut self) -> Result<()> {
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();

    let mut tui = tui::Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
    // tui.mouse(true);
    tui.enter()?;

    for component in self.components.iter_mut() {
      component.register_action_handler(action_tx.clone())?;
    }

    for component in self.components.iter_mut() {
      component.register_config_handler(self.config.clone())?;
    }

    for component in self.components.iter_mut() {
      component.init(tui.size()?)?;
    }

    self.layout_manager.init(tui.size()?)?;

    // main loop
    loop {
      if let Some(e) = tui.next().await {
        match e {
          tui::Event::Quit => action_tx.send(Action::Quit)?,
          tui::Event::Tick => action_tx.send(Action::Tick)?,
          tui::Event::Render => action_tx.send(Action::Render)?,
          tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
          tui::Event::Key(key) => {
            if self.get_focused().scene != Scenes::InputBar {
              // Check global keybinds first
              if let Some(keymap) = self.config.keybindings.get(&Mode::Global) {
                // check for global keybindings
                if let Some(action) = keymap.get(&vec![key]) {
                  log::info!("Got action: {action:?}");
                  action_tx.send(action.clone())?;
                }
              }
              if let Some(keymap) = self.config.keybindings.get(&self.get_focused().mode) {
                if let Some(action) = keymap.get(&vec![key]) {
                  log::info!("Got action: {action:?}");
                  action_tx.send(action.clone())?;
                } else {
                  // If the key was not handled as a single key action,
                  // then consider it for multi-key combinations.
                  self.last_tick_key_events.push(key);

                  // Check for multi-key combinations
                  if let Some(action) = keymap.get(&self.last_tick_key_events) {
                    log::info!("Got action: {action:?}");
                    action_tx.send(action.clone())?;
                  }
                }
              };
            }
          },
          _ => {},
        }
        // send keyboard and mouse inputs to componenets
        let current_focus = self.get_focused();
        for component in self.components.iter_mut() {
          // if in input mode only the input bar and global components will receive inputs
          if current_focus.scene == Scenes::InputBar {
            if component.scene() == Scenes::InputBar {
              if let Some(action) = component.handle_events(
                Some(e.clone()),
                self.focus_buffer.last().expect("focus buffer can never be empty").clone(),
              )? {
                action_tx.send(action)?;
              }
            }
            continue;
          }
          if let Some(action) = component
            .handle_events(Some(e.clone()), self.focus_buffer.last().expect("focus buffer is never empty").clone())?
          {
            action_tx.send(action)?;
          }
        }
      }

      while let Ok(action) = action_rx.try_recv() {
        if action != Action::Tick && action != Action::Render {
          log::debug!("{action:?}");
        }

        // app action handler
        match action {
          Action::Tick => {
            self.last_tick_key_events.drain(..);
          },
          Action::Quit => self.should_quit = true,
          Action::Suspend => self.should_suspend = true,
          Action::Resume => self.should_suspend = false,
          Action::Resize(w, h) => {
            tui.resize(Rect::new(0, 0, w, h))?;
            self.layout_manager.update(tui.size()?)?;
            tui.draw(|f| {
              let current_focus = self.get_focused();
              for component in self.components.iter_mut() {
                let r = component.draw(f, f.size(), current_focus.clone());
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }
              }
            })?;
          },
          Action::Render => {
            tui.draw(|f| {
              let current_mode = self.get_focused().mode;
              let current_focus = self.get_focused();
              for component in self.components.iter_mut() {
                // check if component is to be rendered in the mode or if its a global object
                if component.mode() == current_mode || component.mode() == Mode::Global {
                  match self.layout_manager.get_component_layout(component.scene()) {
                    Ok(layout) => {
                      let r = component.draw(f, layout, current_focus.clone());
                      if let Err(e) = r {
                        action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                      }
                    },
                    // Error and dont render if the scene does not exist
                    Err(e) => action_tx.send(Action::Error(format!("Failed to get layout: {:?}", e))).unwrap(),
                  }
                }
              }
            })?;
          },
          Action::InputModeOn { .. } => {
            self.focus_buffer.push(Focus { mode: self.get_focused().mode, scene: Scenes::InputBar });
          },
          Action::InputModeOff { .. } => {
            self.focus_buffer.pop();
          },
          Action::FocusSwitch(ref focus) => {
            self.focus_buffer.push(focus.clone());
          },
          Action::FocusBack => {
            self.focus_buffer.pop();
          },
          Action::Error(ref error) => error!("error in program: {}", error),
          _ => {},
        }
        // forward actions to components,
        for component in self.components.iter_mut() {
          if let Some(action) = component.update(action.clone())? {
            action_tx.send(action)?
          };
        }
      }
      if self.should_suspend {
        tui.suspend()?;
        action_tx.send(Action::Resume)?;
        tui = tui::Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
        // tui.mouse(true);
        tui.enter()?;
      } else if self.should_quit {
        tui.stop()?;
        break;
      }
    }
    tui.exit()?;
    Ok(())
  }

  fn get_focused(&self) -> Focus {
    self.focus_buffer.last().expect("focus buffer should never be empty").clone()
  }
}
