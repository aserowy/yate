use crate::{
    message::CursorDirection,
    model::{BufferLine, Cursor, CursorPosition, TextBuffer},
};

use super::cursor;

pub fn char(cursor: &mut Cursor, direction: &CursorDirection, model: &TextBuffer) {
    match direction {
        CursorDirection::FindBackward(find) => {
            if let Some(found) = find_char_backward(find, &model.lines, cursor) {
                cursor.horizontal_index = CursorPosition::Absolute {
                    current: found,
                    expanded: found,
                };
            }
        }
        CursorDirection::FindForward(find) => {
            if let Some(found) = find_char_forward(find, &model.lines, cursor) {
                cursor.horizontal_index = CursorPosition::Absolute {
                    current: found,
                    expanded: found,
                };
            }
        }
        CursorDirection::TillBackward(find) => {
            if let Some(found) = find_char_backward(find, &model.lines, cursor) {
                let new = found + 1;
                cursor.horizontal_index = CursorPosition::Absolute {
                    current: new,
                    expanded: new,
                };
            }
        }
        CursorDirection::TillForward(find) => {
            if let Some(found) = find_char_forward(find, &model.lines, cursor) {
                let new = found - 1;
                cursor.horizontal_index = CursorPosition::Absolute {
                    current: new,
                    expanded: new,
                };
            }
        }
        _ => unreachable!(),
    };
}

fn find_char_backward(find: &char, lines: &[BufferLine], cursor: &Cursor) -> Option<usize> {
    let current = match lines.get(cursor.vertical_index) {
        Some(line) => line,
        None => return None,
    };

    let index = match cursor::get_horizontal_index(&cursor.horizontal_index, current) {
        Some(index) => index,
        None => return None,
    };

    if index <= 1 {
        return None;
    }

    current
        .content
        .to_stripped_string()
        .chars()
        .take(index)
        .collect::<Vec<_>>()
        .iter()
        .rposition(|c| c == find)
}

fn find_char_forward(find: &char, lines: &[BufferLine], cursor: &mut Cursor) -> Option<usize> {
    let current = match lines.get(cursor.vertical_index) {
        Some(line) => line,
        None => return None,
    };

    let index = match cursor::get_horizontal_index(&cursor.horizontal_index, current) {
        Some(index) => index,
        None => return None,
    };

    current
        .content
        .to_stripped_string()
        .chars()
        .skip(index + 1)
        .position(|c| &c == find)
        .map(|i| index + i + 1)
}
