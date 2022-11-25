use log::debug;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UiMode {
    Zen,
    Title,
    Help,
    Log,
    TitleHelp,
    TitleLog,
    HelpLog,
    TitleHelpLog,
    Config
}

impl UiMode {
    pub fn to_string(&self) -> String {
        match self {
            UiMode::Zen => "Zen".to_string(),
            UiMode::Title => "Title".to_string(),
            UiMode::Help => "Help".to_string(),
            UiMode::Log => "Log".to_string(),
            UiMode::TitleHelp => "Title and Help".to_string(),
            UiMode::TitleLog => "Title and Log".to_string(),
            UiMode::HelpLog => "Help and Log".to_string(),
            UiMode::TitleHelpLog => "Title, Help and Log".to_string(),
            UiMode::Config => "Config".to_string(),
        }
    }

    pub fn from_number(n: u8) -> UiMode {
        match n {
            1 => UiMode::Zen,
            2 => UiMode::Title,
            3 => UiMode::Help,
            4 => UiMode::Log,
            5 => UiMode::TitleHelp,
            6 => UiMode::TitleLog,
            7 => UiMode::HelpLog,
            8 => UiMode::TitleHelpLog,
            _ => {
                debug!("Invalid UiMode: {}", n);
                UiMode::Title
            }
        }
    }

    pub fn get_available_tabs(&self) -> Vec<String> {
        match self {
            UiMode::Zen => vec!["Body".to_string()],
            UiMode::Title => vec!["Title".to_string(), "Body".to_string()],
            UiMode::Help => vec!["Body".to_string(), "Help".to_string()],
            UiMode::Log => vec!["Body".to_string(), "Log".to_string()],
            UiMode::TitleHelp => vec!["Title".to_string(), "Body".to_string(), "Help".to_string()],
            UiMode::TitleLog => vec!["Title".to_string(), "Body".to_string(), "Log".to_string()],
            UiMode::HelpLog => vec!["Body".to_string(), "Help".to_string(), "Log".to_string()],
            UiMode::TitleHelpLog => vec!["Title".to_string(), "Body".to_string(), "Help".to_string(), "Log".to_string()],
            UiMode::Config => vec!["Config".to_string()],
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum AppState {
    Init,
    Initialized,
    UserInput

}
#[derive(Clone)]
pub enum Focus {
    Title,
    Body,
    Help,
    Log
}

impl AppState {
    pub fn initialized() -> Self {
        Self::Initialized
    }

    pub fn is_initialized(&self) -> bool {
        matches!(self, &Self::Initialized { .. })
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::Init
    }
}

impl Focus {
    pub fn current(&self) -> &str {
        match self {
            Self::Title => "Title",
            Self::Body => "Body",
            Self::Help => "Help",
            Self::Log => "Log",
        }
    }
    pub fn next(&self, available_tabs: &Vec<String>) -> Self {
        let current = self.current();
        let index = available_tabs.iter().position(|x| x == current);
        // check if index is None
        let index = match index {
            Some(i) => i,
            None => 0,
        };
        let next_index = (index + 1) % available_tabs.len();
        match available_tabs[next_index].as_str() {
            "Title" => Self::Title,
            "Body" => Self::Body,
            "Help" => Self::Help,
            "Log" => Self::Log,
            _ => Self::Title,
        }
    }

    pub fn prev(&self, available_tabs: &Vec<String>) -> Self {
        let current = self.current();
        let index = available_tabs.iter().position(|x| x == current);
        // check if index is None
        let index = match index {
            Some(i) => i,
            None => 0,
        };
        let prev_index = if index == 0 {
            available_tabs.len() - 1
        } else {
            index - 1
        };
        match available_tabs[prev_index].as_str() {
            "Title" => Self::Title,
            "Body" => Self::Body,
            "Help" => Self::Help,
            "Log" => Self::Log,
            _ => Self::Title,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Title" => Self::Title,
            "Body" => Self::Body,
            "Help" => Self::Help,
            "Log" => Self::Log,
            _ => Self::Title,
        }
    }
}

impl Default for Focus {
    fn default() -> Self {
        Self::Body
    }
}
