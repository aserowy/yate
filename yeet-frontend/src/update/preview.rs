use yeet_keymap::message::Buffer;

use crate::model::Model;

use super::{buffer, history};

pub fn update(model: &mut Model, message: Option<&Buffer>) {
    let target = &model.preview.path;
    let buffer = &mut model.preview.buffer;
    let layout = &model.layout.preview;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);

    if let Some(message) = message {
        buffer::update(&model.mode, buffer, message);
    } else {
        buffer::reset_view(buffer);
    }

    if !history::set_cursor_index(target, &model.history, buffer) {
        buffer.cursor = None;
    };
}
