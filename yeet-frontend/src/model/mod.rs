use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{error::AppError, settings::Settings};
use ratatui_image::protocol::Protocol;
use tokio_util::sync::CancellationToken;
use yeet_buffer::model::{viewport::ViewPort, Cursor, Mode, TextBuffer};

use self::{history::History, junkyard::JunkYard, mark::Marks, qfix::QuickFix, register::Register};

pub mod history;
pub mod junkyard;
pub mod mark;
pub mod qfix;
pub mod register;

#[derive(Default)]
pub struct Model {
    pub app: App,
    pub settings: Settings,
    pub state: State,
}

pub struct App {
    pub buffers: HashMap<usize, Buffer>,
    pub commandline: CommandLine,
    pub latest_buffer_id: usize,
    pub window: Window,
}

impl Default for App {
    fn default() -> Self {
        let mut buffers = HashMap::new();
        buffers.insert(1, Buffer::FileTree(Box::new(FileTreeBuffer::default())));

        Self {
            buffers,
            commandline: Default::default(),
            latest_buffer_id: 1,
            window: Window::Content(Default::default(), Default::default(), 1),
        }
    }
}

pub enum Window {
    Horizontal(Box<Window>, Box<Window>),
    Content(ViewPort, Cursor, usize),
}

impl Window {
    pub fn get_height(&self) -> Result<u16, AppError> {
        match self {
            Window::Horizontal(_, _) => todo!(),
            Window::Content(vp, _, _) => Ok(vp.height),
        }
    }
}

#[derive(Default)]
pub struct State {
    pub history: History,
    pub junk: JunkYard,
    pub marks: Marks,
    pub modes: ModeState,
    pub qfix: QuickFix,
    pub register: Register,
    pub remaining_keysequence: Option<String>,
    pub tasks: Tasks,
    pub watches: Vec<PathBuf>,
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("settings", &self.settings)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct ModeState {
    pub current: Mode,
    pub previous: Option<Mode>,
}

#[derive(Debug, Default)]
pub struct Tasks {
    pub latest_id: u16,
    pub running: HashMap<String, CurrentTask>,
}

#[derive(Debug)]
pub struct CurrentTask {
    pub external_id: String,
    pub id: u16,
    pub token: CancellationToken,
}

pub struct CommandLine {
    pub buffer: TextBuffer,
    pub cursor: Option<Cursor>,
    pub key_sequence: String,
    pub viewport: ViewPort,
}

impl Default for CommandLine {
    fn default() -> Self {
        Self {
            cursor: Some(Cursor {
                hide_cursor: true,
                hide_cursor_line: true,
                vertical_index: 0,
                ..Default::default()
            }),
            buffer: Default::default(),
            key_sequence: "".to_owned(),
            viewport: Default::default(),
        }
    }
}

pub enum Buffer {
    FileTree(Box<FileTreeBuffer>),
    _Text(Box<TextBuffer>),
}

pub struct FileTreeBuffer {
    pub current: DirectoryBuffer,
    pub parent: FileTreeBufferSectionBuffer,
    pub parent_cursor: Option<Cursor>,
    pub preview: FileTreeBufferSectionBuffer,
    pub preview_cursor: Option<Cursor>,
    pub show_border: bool,
}

impl FileTreeBuffer {
    pub fn get_mut_directories(
        &mut self,
    ) -> Vec<(&Path, &mut ViewPort, &mut Option<Cursor>, &mut TextBuffer)> {
        let parent_content_ref =
            if let FileTreeBufferSectionBuffer::Text(path, buffer) = &mut self.parent {
                Some((
                    path.as_path(),
                    &mut self.parent_vp,
                    &mut self.parent_cursor,
                    buffer,
                ))
            } else {
                None
            };

        let preview_content_ref =
            if let FileTreeBufferSectionBuffer::Text(path, buffer) = &mut self.preview {
                Some((
                    path.as_path(),
                    &mut self.preview_vp,
                    &mut self.preview_cursor,
                    buffer,
                ))
            } else {
                None
            };

        vec![
            Some((
                self.current.path.as_path(),
                &mut self.current_vp,
                &mut self.current_cursor,
                &mut self.current.buffer,
            )),
            parent_content_ref,
            preview_content_ref,
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
    }
}

impl Default for FileTreeBuffer {
    fn default() -> Self {
        Self {
            current: Default::default(),
            parent: Default::default(),
            parent_cursor: Default::default(),
            preview: Default::default(),
            preview_cursor: Default::default(),
            show_border: true,
        }
    }
}

#[derive(Debug)]
pub enum FileTreeBufferSection {
    Current,
    Parent,
    Preview,
}

// NOTE: most of the time Text is used. Thus, boxing Buffer would only increase hassle to work
// with this BufferType.
#[allow(clippy::large_enum_variant)]
#[derive(Default)]
pub enum FileTreeBufferSectionBuffer {
    Image(PathBuf, Protocol),
    #[default]
    None,
    Text(PathBuf, TextBuffer),
}

impl FileTreeBufferSectionBuffer {
    pub fn resolve_path(&self) -> Option<&Path> {
        match self {
            FileTreeBufferSectionBuffer::Text(path, _) => Some(path),
            FileTreeBufferSectionBuffer::Image(path, _) => Some(path),
            FileTreeBufferSectionBuffer::None => None,
        }
    }
}

#[derive(Default)]
pub struct DirectoryBuffer {
    pub buffer: TextBuffer,
    pub path: PathBuf,
    pub state: DirectoryBufferState,
}

#[derive(Debug, Default, PartialEq)]
pub enum DirectoryBufferState {
    Loading,
    PartiallyLoaded,
    Ready,
    #[default]
    Uninitialized,
}
