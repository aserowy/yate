use std::mem;

use crate::{
    message::{CursorDirection, LineDirection, TextModification},
    model::{
        ansi::Ansi, undo::BufferChanged, BufferLine, Cursor, CursorPosition, Mode, TextBuffer,
    },
};

use super::cursor;

pub fn update(
    cursor: &mut Cursor,
    mode: &Mode,
    buffer: &mut TextBuffer,
    count: &usize,
    modification: &TextModification,
) -> Option<Vec<BufferChanged>> {
    match modification {
        TextModification::DeleteLine => {
            if buffer.lines.is_empty() {
                return None;
            }

            let mut changes = Vec::new();
            for _ in 0..*count {
                if buffer.lines.is_empty() {
                    break;
                }

                let line_index = cursor.vertical_index;
                let line = buffer.lines.remove(line_index);

                let line_count = buffer.lines.len();
                if line_count == 0 {
                    cursor.vertical_index = 0;
                } else if line_index >= line_count {
                    cursor.vertical_index = line_count - 1;
                }

                changes.push(BufferChanged::LineRemoved(line_index, line.content));
            }

            cursor::set_to_inbound_position(cursor, mode, buffer);

            Some(changes)
        }
        TextModification::DeleteMotion(delete_count, motion) => {
            let pre_motion_cursor = cursor.clone();
            for _ in 0..*count {
                cursor::update_by_direction(cursor, mode, buffer, delete_count, motion);
            }

            let mut changes = Vec::new();
            if is_line_delete(motion) {
                let pre_index = pre_motion_cursor.vertical_index;
                let post_index = cursor.vertical_index;

                if pre_index == post_index {
                    return None;
                }

                let count = if pre_index > post_index {
                    pre_index - post_index + 1
                } else {
                    let _ = mem::replace(cursor, pre_motion_cursor.clone());
                    post_index - pre_index + 1
                };

                let action = &TextModification::DeleteLine;
                if let Some(cng) = update(cursor, mode, buffer, &count, action) {
                    changes.extend(cng);
                }
            } else {
                // TODO: multi line motion like search
                let line = match buffer.lines.get_mut(pre_motion_cursor.vertical_index) {
                    Some(it) => it,
                    None => return None,
                };

                let pre_index = get_cursor_index(&pre_motion_cursor, line);

                let post_index = match pre_motion_cursor.vertical_index.cmp(&cursor.vertical_index)
                {
                    std::cmp::Ordering::Greater => 0,
                    std::cmp::Ordering::Less => line.content.count_chars() - 1,
                    std::cmp::Ordering::Equal => get_cursor_index(cursor, line),
                };

                let (index, mut count) = if pre_index > post_index {
                    (post_index, pre_index - post_index)
                } else {
                    let _ = mem::replace(cursor, pre_motion_cursor.clone());
                    (pre_index, post_index - pre_index)
                };

                if is_inclusive(motion) {
                    count += 1;
                }

                let mut modified = line.content.clone();
                modified.remove(index, count);

                let changed = BufferChanged::Content(
                    pre_motion_cursor.vertical_index,
                    line.content.clone(),
                    modified.clone(),
                );

                line.content = modified;

                changes.push(changed);
            }

            cursor::set_to_inbound_position(cursor, mode, buffer);

            Some(changes)
        }
        TextModification::Insert(raw) => {
            let line = get_line_or_create_on_empty(cursor, buffer);
            if let Some((cursor, line)) = line {
                let index = get_cursor_index(cursor, line);

                let next_index = index + raw.chars().count();
                cursor.horizontal_index = CursorPosition::Absolute {
                    current: next_index,
                    expanded: next_index,
                };

                let mut new = line.content.clone();
                new.insert(index, raw);

                let changed = BufferChanged::Content(
                    cursor.vertical_index,
                    line.content.clone(),
                    new.clone(),
                );
                line.content = new;

                Some(vec![changed])
            } else {
                None
            }
        }
        TextModification::InsertNewLine(direction) => {
            let index = match direction {
                LineDirection::Up => cursor.vertical_index,
                LineDirection::Down => {
                    if buffer.lines.is_empty() {
                        cursor.vertical_index = 0;

                        0
                    } else {
                        let index = cursor.vertical_index + 1;
                        cursor.vertical_index = index;

                        index
                    }
                }
            };

            cursor.horizontal_index = CursorPosition::Absolute {
                current: 0,
                expanded: 0,
            };

            buffer.lines.insert(index, BufferLine::default());

            Some(vec![BufferChanged::LineAdded(index, Ansi::new(""))])
        }
        TextModification::InsertLineBreak => {
            let line = get_line_or_create_on_empty(cursor, buffer);

            if let Some((cursor, line)) = line {
                let horizontal = get_cursor_index(cursor, line);

                let renamed = line.content.take_chars(horizontal);
                let new = line.content.skip_chars(horizontal);

                let mut changed = Vec::new();
                if line.content != renamed {
                    changed.push(BufferChanged::Content(
                        cursor.vertical_index,
                        line.content.clone(),
                        renamed.clone(),
                    ));
                    line.content = renamed;
                }

                cursor.horizontal_index = CursorPosition::Absolute {
                    current: 0,
                    expanded: 0,
                };

                let vertical = cursor.vertical_index + 1;
                cursor.vertical_index = vertical;

                buffer.lines.insert(
                    vertical,
                    BufferLine {
                        content: new.clone(),
                        ..Default::default()
                    },
                );

                changed.push(BufferChanged::LineAdded(vertical, new));

                Some(changed)
            } else {
                None
            }
        }
    }
}

fn is_inclusive(motion: &CursorDirection) -> bool {
    match motion {
        CursorDirection::Left
        | CursorDirection::Right
        | CursorDirection::LineStart
        | CursorDirection::Search(_)
        | CursorDirection::Up
        | CursorDirection::Down
        | CursorDirection::Bottom
        | CursorDirection::WordStartBackward
        | CursorDirection::WordStartForward
        | CursorDirection::WordUpperStartBackward
        | CursorDirection::WordUpperStartForward
        | CursorDirection::Top => false,

        CursorDirection::FindBackward(_)
        | CursorDirection::FindForward(_)
        | CursorDirection::TillBackward(_)
        | CursorDirection::TillForward(_)
        | CursorDirection::LastFindBackward
        | CursorDirection::LastFindForward
        | CursorDirection::WordEndBackward
        | CursorDirection::WordEndForward
        | CursorDirection::WordUpperEndBackward
        | CursorDirection::WordUpperEndForward
        | CursorDirection::LineEnd => true,
    }
}

fn is_line_delete(motion: &CursorDirection) -> bool {
    match motion {
        CursorDirection::Up
        | CursorDirection::Down
        | CursorDirection::Bottom
        | CursorDirection::Top => true,

        CursorDirection::FindBackward(_)
        | CursorDirection::FindForward(_)
        | CursorDirection::TillBackward(_)
        | CursorDirection::TillForward(_)
        | CursorDirection::LastFindBackward
        | CursorDirection::LastFindForward
        | CursorDirection::Left
        | CursorDirection::Right
        | CursorDirection::Search(_)
        | CursorDirection::LineEnd
        | CursorDirection::WordEndBackward
        | CursorDirection::WordEndForward
        | CursorDirection::WordStartBackward
        | CursorDirection::WordStartForward
        | CursorDirection::WordUpperEndBackward
        | CursorDirection::WordUpperEndForward
        | CursorDirection::WordUpperStartBackward
        | CursorDirection::WordUpperStartForward
        | CursorDirection::LineStart => false,
    }
}

fn get_line_or_create_on_empty<'a>(
    cursor: &'a mut Cursor,
    model: &'a mut TextBuffer,
) -> Option<(&'a mut Cursor, &'a mut BufferLine)> {
    if cursor.horizontal_index == CursorPosition::None {
        return None;
    }

    if model.lines.is_empty() {
        cursor.vertical_index = 0;

        let line = BufferLine::default();
        model.lines.push(line);

        Some((cursor, &mut model.lines[0]))
    } else {
        let line = match model.lines.get_mut(cursor.vertical_index) {
            Some(it) => it,
            None => return None,
        };

        Some((cursor, line))
    }
}

fn get_cursor_index(cursor: &Cursor, line: &BufferLine) -> usize {
    match cursor.horizontal_index {
        CursorPosition::Absolute {
            current,
            expanded: _,
        } => current,
        CursorPosition::End => {
            if line.is_empty() {
                0
            } else {
                line.len() - 1
            }
        }
        CursorPosition::None => unreachable!(),
    }
}
