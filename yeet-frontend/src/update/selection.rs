use std::path::PathBuf;

use yeet_buffer::model::BufferLine;

use crate::{
    action::Action,
    event::Message,
    model::{register::Register, FileTreeBuffer},
};

pub fn get_current_selected_path(buffer: &FileTreeBuffer) -> Option<PathBuf> {
    let current_buffer = &buffer.current.buffer;
    if current_buffer.lines.is_empty() {
        return None;
    }

    let cursor = &buffer.current_cursor.as_ref()?;
    let current = &current_buffer.lines.get(cursor.vertical_index)?;
    if current.content.is_empty() {
        return None;
    }

    let target = buffer
        .current
        .path
        .join(current.content.to_stripped_string());

    if target.exists() {
        Some(target)
    } else {
        None
    }
}

pub fn get_current_selected_bufferline(buffer: &mut FileTreeBuffer) -> Option<&mut BufferLine> {
    let current_buffer = &mut buffer.current.buffer;
    if current_buffer.lines.is_empty() {
        return None;
    }

    let cursor = &buffer.current_cursor.as_ref()?;
    current_buffer.lines.get_mut(cursor.vertical_index)
}

pub fn copy_to_clipboard(register: &mut Register, buffer: &FileTreeBuffer) -> Vec<Action> {
    if let Some(path) = get_current_selected_path(buffer) {
        if let Some(clipboard) = register.clipboard.as_mut() {
            match clipboard.set_text(path.to_string_lossy()) {
                Ok(_) => Vec::new(),
                Err(err) => vec![Action::EmitMessages(vec![Message::Error(err.to_string())])],
            }
        } else {
            vec![Action::EmitMessages(vec![Message::Error(
                "Clipboard not available".to_string(),
            )])]
        }
    } else {
        Vec::new()
    }
}
