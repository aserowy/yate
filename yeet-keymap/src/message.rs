use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum Binding {
    Message(Message),
    Mode(Mode),
    ModeAndNotRepeatedMotion(Mode, CursorDirection),
    ModeAndTextModification(Mode, TextModification),
    Motion(CursorDirection),
    Repeat(usize),
    RepeatOrMotion(usize, CursorDirection),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    Buffer(Buffer),
    ExecuteCommand,
    ExecuteCommandString(String),
    KeySequenceChanged(String),
    NavigateToParent,
    NavigateToPath(PathBuf),
    NavigateToSelected,
    OpenSelected,
    PasteRegister(String),
    EnumerationChanged(PathBuf, Vec<(ContentKind, String)>),
    EnumerationFinished(PathBuf),
    PathRemoved(PathBuf),
    PathsAdded(Vec<PathBuf>),
    PathsWriteFinished(Vec<PathBuf>),
    PreviewLoaded(PathBuf, Vec<String>),
    Print(Vec<PrintContent>),
    Rerender,
    Resize(u16, u16),
    Quit,
    YankSelected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrintContent {
    Error(String),
    Info(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentKind {
    Directory,
    File,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Buffer {
    ChangeMode(Mode, Mode),
    Modification(TextModification),
    MoveCursor(usize, CursorDirection),
    MoveViewPort(ViewPortDirection),
    SaveBuffer(Option<usize>),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TextModification {
    DeleteCharBeforeCursor,
    DeleteCharOnCursor,
    DeleteLineOnCursor,
    Insert(String),
    InsertNewLine(NewLineDirection),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NewLineDirection {
    Above,
    Under,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CursorDirection {
    Bottom,
    Down,
    Left,
    LineEnd,
    LineStart,
    Right,
    Top,
    Up,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Mode {
    Command,
    Insert,

    #[default]
    Navigation,

    Normal,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Command => "command".to_string(),
            Mode::Insert => "insert".to_string(),
            Mode::Navigation => "navigation".to_string(),
            Mode::Normal => "normal".to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViewPortDirection {
    BottomOnCursor,
    CenterOnCursor,
    HalfPageDown,
    HalfPageUp,
    TopOnCursor,
}
