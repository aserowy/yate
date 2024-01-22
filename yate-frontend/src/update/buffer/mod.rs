use yate_keymap::message::Message;

use crate::model::buffer::{Buffer, Cursor, CursorPosition};

mod cursor;
mod viewport;

pub fn update(model: &mut Buffer, message: &Message) {
    match message {
        Message::ChangeKeySequence(_) => {}
        Message::ChangeMode(_) => {}
        Message::MoveCursor(count, direction) => {
            cursor::update_by_direction(model, count, direction);
            viewport::update_by_cursor(model);
        }
        Message::MoveViewPort(direction) => viewport::update_by_direction(model, direction),
        Message::Refresh => {}
        Message::SelectCurrent => reset_cursor(&mut model.cursor),
        Message::SelectParent => reset_cursor(&mut model.cursor),
        Message::Quit => {}
    }
}

fn reset_cursor(cursor: &mut Option<Cursor>) {
    if let Some(cursor) = cursor {
        cursor.vertical_index = 0;
        cursor.horizontial_index = match &cursor.horizontial_index {
            CursorPosition::Absolute(_) => CursorPosition::Absolute(0),
            CursorPosition::End => CursorPosition::End,
            CursorPosition::None => CursorPosition::None,
        }
    }
}
