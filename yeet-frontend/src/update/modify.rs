use yeet_buffer::{
    message::{BufferMessage, TextModification},
    model::Mode,
};

use crate::{
    action::Action,
    model::{App, Buffer, FileTreeBufferSectionBuffer},
};

use super::app;

pub fn buffer(
    app: &mut App,
    mode: &Mode,
    repeat: &usize,
    modification: &TextModification,
) -> Vec<Action> {
    let msg = BufferMessage::Modification(*repeat, modification.clone());
    match app::get_focused_mut(app) {
        (vp, cursor, Buffer::FileTree(it)) => {
            yeet_buffer::update(vp, Some(cursor), mode, &mut it.current.buffer, &msg);

            // FIX: only if selection changed!
            it.preview = FileTreeBufferSectionBuffer::None;
        }
        (vp, cursor, Buffer::_Text(_)) => todo!(),
    }
    Vec::new()
}
