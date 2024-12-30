use message::BufferMessage;
use model::{viewport::ViewPort, BufferResult, Cursor, Mode, TextBuffer};
use ratatui::Frame;

pub mod message;
pub mod model;
mod update;
mod view;

pub fn update(
    viewport: Option<&ViewPort>,
    cursor: Option<&Cursor>,
    mode: &Mode,
    buffer: &mut TextBuffer,
    messages: Vec<&BufferMessage>,
) -> (Option<ViewPort>, Option<Cursor>, Vec<BufferResult>) {
    update::update(viewport, cursor, mode, buffer, messages)
}

pub fn view(
    viewport: &ViewPort,
    cursor: Option<&Cursor>,
    mode: &Mode,
    buffer: &TextBuffer,
    frame: &mut Frame,
    horizontal_offset: u16,
    vertical_offset: u16,
) {
    view::view(
        viewport,
        cursor,
        mode,
        buffer,
        frame,
        horizontal_offset,
        vertical_offset,
    )
}
