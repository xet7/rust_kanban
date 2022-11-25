use std::path::PathBuf;

use log::debug;
use log::{error, warn};
use serde::{Serialize, Deserialize};

use self::actions::Actions;
use self::state::AppState;
use self::state::Focus;
use self::state::UiMode;
use self::kanban::Board;
use crate::app::actions::Action;
use crate::constants::DB_NAME;
use crate::inputs::key::Key;
use crate::io::{IoEvent, handler, data_handler};

pub mod actions;
pub mod state;
pub mod ui;
pub mod kanban;

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

/// The main application, containing the state
pub struct App {
    /// We could dispatch an IO event
    io_tx: tokio::sync::mpsc::Sender<IoEvent>,
    /// Contextual actions
    actions: Actions,
    /// State
    is_loading: bool,
    state: AppState,
    focus: Focus,
    ui_mode: UiMode,
    boards: Vec<kanban::Board>,
    go_up: bool,
    go_down: bool,
    current_obj_index: usize,
    current_user_input: String,
    prev_ui_mode: UiMode,
}

impl App {
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> Self {
        let actions = vec![Action::Quit].into();
        let is_loading = false;
        let state = AppState::default();
        let focus = Focus::Title;
        let ui_mode = data_handler::get_default_ui_mode();
        let boards = vec![];
        let go_up = false;
        let go_down = false;
        let current_obj_index = 0;

        Self {
            io_tx,
            actions,
            is_loading,
            state,
            focus,
            ui_mode,
            go_up,
            go_down,
            boards: boards,
            current_obj_index,
            current_user_input: String::new(),
            prev_ui_mode: UiMode::Zen,
        }
    }

    /// Handle a user action
    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        // check if we are in a user input mode
        if self.state == AppState::UserInput {
            debug!("User input mode");
            // append to current user input if key is not enter else change state to Initialized
            if key != Key::Enter {
                self.current_user_input.push_str(&key.to_string());
            } else {
                self.state = AppState::Initialized;
                debug!("User input: {}", self.current_user_input);
            }
            return AppReturn::Continue;
        } else {
            if let Some(action) = self.actions.find(key) {
                match action {
                    Action::Quit => AppReturn::Exit,
                    // Action::Sleep => {
                    //     if let Some(duration) = self.state.duration().cloned() {
                    //         // Sleep is an I/O action, we dispatch on the IO channel that's run on another thread
                    //         self.dispatch(IoEvent::Sleep(duration)).await
                    //     }
                    //     AppReturn::Continue
                    // }
                    Action::NextFocus => {
                        self.focus = self.focus.next(&UiMode::get_available_tabs(&self.ui_mode));
                        AppReturn::Continue
                    }
                    Action::PreviousFocus => {
                        self.focus = self.focus.prev(&UiMode::get_available_tabs(&self.ui_mode));
                        AppReturn::Continue
                    }
                    Action::SetUiMode => {
                        let new_ui_mode = UiMode::from_number(key.to_digit() as u8);
                        let available_tabs = UiMode::get_available_tabs(&new_ui_mode);
                        // check if focus is still available in the new ui_mode if not set it to the first available tab
                        if !available_tabs.contains(&self.focus.current().to_owned()) {
                            self.focus = Focus::from_str(available_tabs[0].as_str());
                        }
                        debug!("Setting ui_mode to {}", new_ui_mode.to_string());
                        self.ui_mode = new_ui_mode;
                        AppReturn::Continue
                    }
                    Action::ToggleConfig => {
                        if self.ui_mode == UiMode::Config {
                            self.ui_mode = self.prev_ui_mode.clone();
                            self.current_obj_index = 0;
                        } else {
                            self.prev_ui_mode = self.ui_mode.clone();
                            self.ui_mode = UiMode::Config;
                        }
                        AppReturn::Continue
                    }
                    Action::GoUp => {
                        if self.ui_mode == UiMode::Config {
                            self.go_up = true;
                        }
                        AppReturn::Continue
                    }
                    Action::GoDown => {
                        if self.ui_mode == UiMode::Config {
                            self.go_down = true;
                        }
                        AppReturn::Continue
                    }
                    Action::TakeUserInput => {
                        debug!("Taking user input");
                        self.state = AppState::UserInput;
                        AppReturn::Continue
                    }
                }
            } else {
                warn!("No action accociated to {}", key);
                AppReturn::Continue
            }
        }
    }

    /// Send a network event to the IO thread
    pub async fn dispatch(&mut self, action: IoEvent) {
        // `is_loading` will be set to false again after the async action has finished in io/handler.rs
        self.is_loading = true;
        if let Err(e) = self.io_tx.send(action).await {
            self.is_loading = false;
            error!("Error from dispatch {}", e);
        };
    }

    pub fn actions(&self) -> &Actions {
        &self.actions
    }
    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    pub fn initialized(&mut self) {
        // Update contextual actions
        self.actions = vec![
            Action::Quit,
            Action::NextFocus,
            Action::PreviousFocus,
            Action::SetUiMode,
            Action::ToggleConfig,
            Action::GoUp,
            Action::GoDown,
            Action::TakeUserInput,
        ]
        .into();
        self.state = AppState::initialized()
    }

    pub fn set_boards(&mut self, boards: Vec<Board>) {
        self.boards = boards;
    }

    pub fn loaded(&mut self) {
        self.is_loading = false;
    }

    pub fn current_focus(&self) -> &Focus {
        &self.focus
    }

    pub fn change_focus(&mut self, focus: Focus) {
        self.focus = focus;
    }

    pub fn set_current_user_input(&mut self, input: String) {
        let new_input = input;
        debug!("Setting current user input to {}", new_input);
        self.current_user_input = new_input;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub db_path: PathBuf,
    pub default_view: UiMode
}

impl AppConfig {
    pub fn default() -> Self {
        let db_path = handler::get_config_dir().join(DB_NAME);
        let default_view = UiMode::TitleHelpLog;
        Self {
            db_path,
            default_view
        }
    }

    pub fn to_list(&self) -> Vec<String> {
        vec![
            format!("db_path: {}", self.db_path.to_str().unwrap()),
            format!("default_view: {}", self.default_view.to_string()),
        ]
    }
}
