use crate::{
    message::ViewPortDirection,
    model::{viewport::ViewPort, Cursor, CursorPosition, TextBuffer},
};

pub fn update_by_cursor(viewport: &mut ViewPort, cursor: &Cursor, buffer: &TextBuffer) {
    if buffer.lines.is_empty() {
        return;
    }

    let viewport_offset = if viewport.height == 0 {
        0
    } else {
        usize::from(viewport.height - 1)
    };

    if viewport.vertical_index > cursor.vertical_index {
        viewport.vertical_index = cursor.vertical_index;
    } else if viewport.vertical_index + viewport_offset < cursor.vertical_index {
        viewport.vertical_index = cursor.vertical_index - viewport_offset;
    }

    let cursor_index = match cursor.horizontal_index {
        CursorPosition::Absolute {
            current,
            expanded: _,
        } => current,
        CursorPosition::End => {
            let line_lenght = buffer.lines[cursor.vertical_index].len();
            if line_lenght == 0 {
                0
            } else {
                line_lenght - 1
            }
        }
        CursorPosition::None => return,
    };

    let line = &buffer.lines[cursor.vertical_index];
    if viewport.horizontal_index > cursor_index {
        viewport.horizontal_index = cursor_index;
    } else if viewport.horizontal_index + viewport.get_content_width(line) < cursor_index {
        viewport.horizontal_index = cursor_index - viewport.get_content_width(line)
    }
}

pub fn update_by_direction(
    viewport: &mut ViewPort,
    cursor: &mut Option<Cursor>,
    buffer: &TextBuffer,
    direction: &ViewPortDirection,
) {
    if buffer.lines.is_empty() {
        return;
    }

    match direction {
        ViewPortDirection::BottomOnCursor => {
            if let Some(cursor) = &cursor {
                if cursor.vertical_index < usize::from(viewport.height) {
                    viewport.vertical_index = 0;
                } else {
                    let index = cursor.vertical_index - usize::from(viewport.height) + 1;
                    viewport.vertical_index = index;
                }
            }
        }
        ViewPortDirection::CenterOnCursor => {
            if let Some(cursor) = &cursor {
                let index_offset = viewport.height / 2;
                if cursor.vertical_index < usize::from(index_offset) {
                    viewport.vertical_index = 0;
                } else {
                    viewport.vertical_index = cursor.vertical_index - usize::from(index_offset);
                }
            }
        }
        ViewPortDirection::HalfPageDown => {
            let index_offset = viewport.height / 2;
            let viewport_end_index = viewport.vertical_index + usize::from(viewport.height - 1);
            let viewport_end_after_move_index = viewport_end_index + usize::from(index_offset);

            if viewport_end_after_move_index < buffer.lines.len() {
                viewport.vertical_index += usize::from(index_offset);
            } else if usize::from(viewport.height) > buffer.lines.len() {
                viewport.vertical_index = 0;
            } else {
                viewport.vertical_index = buffer.lines.len() - usize::from(viewport.height);
            }

            if let Some(cursor) = cursor {
                if cursor.vertical_index + usize::from(index_offset) >= buffer.lines.len() {
                    cursor.vertical_index = buffer.lines.len() - 1;
                } else {
                    cursor.vertical_index += usize::from(index_offset);
                }
            }
        }
        ViewPortDirection::HalfPageUp => {
            let index_offset = viewport.height / 2;
            if viewport.vertical_index < usize::from(index_offset) {
                viewport.vertical_index = 0;
            } else {
                viewport.vertical_index -= usize::from(index_offset);
            }

            if let Some(cursor) = cursor {
                if cursor.vertical_index < usize::from(index_offset) {
                    cursor.vertical_index = 0;
                } else {
                    cursor.vertical_index -= usize::from(index_offset);
                }
            }
        }
        ViewPortDirection::TopOnCursor => {
            if let Some(cursor) = &cursor {
                viewport.vertical_index = cursor.vertical_index;
            }
        }
    }
}
