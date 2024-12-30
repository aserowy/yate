use std::fmt::Display;

use crate::message::CursorDirection;

use ansi::Ansi;
use undo::{BufferChanged, Undo};

pub mod ansi;
pub mod undo;
pub mod viewport;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Mode {
    Command(CommandMode),
    Insert,
    #[default]
    Navigation,
    Normal,
}

impl Mode {
    pub fn is_command(&self) -> bool {
        matches!(self, Mode::Command(_))
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            Mode::Command(_) => "command".to_string(),
            Mode::Insert => "insert".to_string(),
            Mode::Navigation => "navigation".to_string(),
            Mode::Normal => "normal".to_string(),
        };

        write!(f, "{}", content)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CommandMode {
    Command,
    PrintMultiline,
    Search(SearchDirection),
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum SearchDirection {
    #[default]
    Down,
    Up,
}

#[derive(Default)]
pub struct TextBuffer {
    pub last_find: Option<CursorDirection>,
    pub lines: Vec<BufferLine>,
    pub undo: Undo,
}

impl std::fmt::Debug for TextBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("last_find", &self.last_find)
            .field("lines", &self.lines)
            .finish()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Cursor {
    pub hide_cursor: bool,
    pub hide_cursor_line: bool,
    pub horizontal_index: CursorPosition,
    pub vertical_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CursorPosition {
    Absolute { current: usize, expanded: usize },
    End,
    None,
}

impl Default for CursorPosition {
    fn default() -> Self {
        CursorPosition::Absolute {
            current: 0,
            expanded: 0,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BufferLine {
    pub prefix: Option<String>,
    pub content: Ansi,
    pub search_char_position: Option<Vec<(usize, usize)>>,
    pub signs: Vec<Sign>,
}

impl BufferLine {
    pub fn from(content: &str) -> Self {
        Self {
            content: Ansi::new(content),
            ..Default::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn len(&self) -> usize {
        self.content.count_chars()
    }
}

pub type SignIdentifier = &'static str;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sign {
    pub id: SignIdentifier,
    pub content: char,
    pub priority: usize,
    pub style: String,
}

#[derive(Clone, PartialEq)]
pub enum BufferResult {
    Changes(Vec<BufferChanged>),
    CursorPositionChanged,
    FindScopeChanged(CursorDirection),
}
