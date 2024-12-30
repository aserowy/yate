use std::cmp::Ordering;

use crate::{
    message::{CursorDirection, Search},
    model::{BufferLine, BufferResult, Cursor, CursorPosition, Mode, TextBuffer},
};

use super::{find, word};

// TODO: refactor
pub fn update_by_direction(
    cursor: &mut Cursor,
    mode: &Mode,
    buffer: &mut TextBuffer,
    count: &usize,
    direction: &CursorDirection,
) -> Vec<BufferResult> {
    if buffer.lines.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();
    for _ in 0..*count {
        match direction {
            CursorDirection::Bottom => {
                cursor.vertical_index = buffer.lines.len() - 1;
                let line = match buffer.lines.get(cursor.vertical_index) {
                    Some(line) => line,
                    None => return Vec::new(),
                };

                let position = get_position(mode, &line.len(), &cursor.horizontal_index);
                cursor.horizontal_index = position;
            }
            CursorDirection::Down => {
                let max_index = buffer.lines.len() - 1;
                if cursor.vertical_index == max_index {
                    return Vec::new();
                }

                if cursor.vertical_index > max_index {
                    cursor.vertical_index = max_index;
                } else {
                    cursor.vertical_index += 1
                }

                let line = match buffer.lines.get(cursor.vertical_index) {
                    Some(line) => line,
                    None => return Vec::new(),
                };

                let line_length = &line.len();
                let position = get_position(mode, line_length, &cursor.horizontal_index);

                cursor.horizontal_index = position;
            }
            CursorDirection::FindBackward(_)
            | CursorDirection::FindForward(_)
            | CursorDirection::TillBackward(_)
            | CursorDirection::TillForward(_) => {
                find::char(cursor, direction, buffer);
                buffer.last_find = Some(direction.clone());

                results.push(BufferResult::FindScopeChanged(direction.clone()));
            }
            CursorDirection::LastFindBackward => {
                if let Some(find) = buffer.last_find.clone() {
                    let find = match find {
                        CursorDirection::FindBackward(find) => CursorDirection::FindForward(find),
                        CursorDirection::FindForward(find) => CursorDirection::FindBackward(find),
                        CursorDirection::TillBackward(find) => CursorDirection::TillForward(find),
                        CursorDirection::TillForward(find) => CursorDirection::TillBackward(find),
                        _ => unreachable!(),
                    };

                    find::char(cursor, &find, buffer);
                }
            }
            CursorDirection::LastFindForward => {
                if let Some(find) = buffer.last_find.clone() {
                    find::char(cursor, &find, buffer);
                }
            }
            CursorDirection::Left => {
                let line = match buffer.lines.get(cursor.vertical_index) {
                    Some(line) => line,
                    None => return Vec::new(),
                };

                let index = match get_horizontal_index(&cursor.horizontal_index, line) {
                    Some(index) => index,
                    None => return Vec::new(),
                };

                if index > 0 {
                    let next_index = index - 1;

                    cursor.horizontal_index = CursorPosition::Absolute {
                        current: next_index,
                        expanded: next_index,
                    };
                }
            }
            CursorDirection::LineEnd => {
                if mode == &Mode::Insert {
                    let line = match buffer.lines.get(cursor.vertical_index) {
                        Some(line) => line,
                        None => return Vec::new(),
                    };

                    let index_correction = get_index_correction(mode);
                    let max_index = line.len() - index_correction;

                    cursor.horizontal_index = CursorPosition::Absolute {
                        current: max_index,
                        expanded: max_index,
                    };
                } else {
                    cursor.horizontal_index = CursorPosition::End;
                }
            }
            CursorDirection::LineStart => {
                cursor.horizontal_index = CursorPosition::Absolute {
                    current: 0,
                    expanded: 0,
                };
            }
            CursorDirection::Right => {
                let line = match buffer.lines.get(cursor.vertical_index) {
                    Some(line) => line,
                    None => return Vec::new(),
                };

                let index_correction = get_index_correction(mode);
                let max_index = line.len() - index_correction;

                let cursor_index = match cursor.horizontal_index {
                    CursorPosition::Absolute {
                        current,
                        expanded: _,
                    } => current,
                    CursorPosition::End => {
                        if mode == &Mode::Insert {
                            // NOTE: -1 to trigger replacement with absolute cursor
                            max_index - 1
                        } else {
                            return Vec::new();
                        }
                    }
                    CursorPosition::None => return Vec::new(),
                };

                if max_index > cursor_index {
                    let next_index = cursor_index + 1;

                    cursor.horizontal_index = CursorPosition::Absolute {
                        current: next_index,
                        expanded: next_index,
                    };
                }
            }
            CursorDirection::Search(direction) => {
                jump_to_next_search(cursor, &buffer.lines, direction);
            }
            CursorDirection::Top => {
                cursor.vertical_index = 0;

                let line = match buffer.lines.get(cursor.vertical_index) {
                    Some(line) => line,
                    None => return Vec::new(),
                };

                let line_length = &line.len();
                let position = get_position(mode, line_length, &cursor.horizontal_index);

                cursor.horizontal_index = position;
            }
            CursorDirection::Up => {
                if cursor.vertical_index == 0 {
                    return Vec::new();
                }

                cursor.vertical_index -= 1;

                let line = match buffer.lines.get(cursor.vertical_index) {
                    Some(line) => line,
                    None => return Vec::new(),
                };

                let line_length = &line.len();
                let position = get_position(mode, line_length, &cursor.horizontal_index);

                cursor.horizontal_index = position;
            }
            CursorDirection::WordEndBackward => {
                word::move_cursor_to_word_end_backward(cursor, buffer, false);
            }
            CursorDirection::WordEndForward => {
                word::move_cursor_to_word_end_forward(cursor, buffer, false);
            }
            CursorDirection::WordStartBackward => {
                word::move_cursor_to_word_start_backward(cursor, buffer, false);
            }
            CursorDirection::WordStartForward => {
                word::move_cursor_to_word_start_forward(cursor, buffer, false);
            }
            CursorDirection::WordUpperEndBackward => {
                word::move_cursor_to_word_end_backward(cursor, buffer, true);
            }
            CursorDirection::WordUpperEndForward => {
                word::move_cursor_to_word_end_forward(cursor, buffer, true);
            }
            CursorDirection::WordUpperStartBackward => {
                word::move_cursor_to_word_start_backward(cursor, buffer, true);
            }
            CursorDirection::WordUpperStartForward => {
                word::move_cursor_to_word_start_forward(cursor, buffer, true);
            }
        }
    }

    results
}

pub fn set_to_inbound_position(
    cursor: &mut Cursor,
    mode: &Mode,
    model: &TextBuffer,
) {
    let position = if model.lines.is_empty() {
        get_position(mode, &0, &cursor.horizontal_index)
    } else {
        let max_index = model.lines.len() - 1;
        if cursor.vertical_index >= max_index {
            cursor.vertical_index = max_index;
        }

        let line = match model.lines.get(cursor.vertical_index) {
            Some(line) => line,
            None => return,
        };

        let line_length = &line.len();
        get_position(mode, line_length, &cursor.horizontal_index)
    };

    cursor.horizontal_index = position;
}

fn get_position(mode: &Mode, line_length: &usize, position: &CursorPosition) -> CursorPosition {
    match position {
        CursorPosition::Absolute {
            current: _,
            expanded,
        } => {
            let index_correction = get_index_correction(mode);
            let max_length = if line_length == &0 {
                index_correction
            } else {
                line_length - index_correction
            };

            if expanded > &max_length {
                CursorPosition::Absolute {
                    current: max_length,
                    expanded: *expanded,
                }
            } else {
                CursorPosition::Absolute {
                    current: *expanded,
                    expanded: *expanded,
                }
            }
        }
        CursorPosition::End => CursorPosition::End,
        CursorPosition::None => CursorPosition::None,
    }
}

fn get_index_correction(mode: &Mode) -> usize {
    match mode {
        Mode::Command(_) => 0,
        Mode::Insert => 0,
        Mode::Navigation => 1,
        Mode::Normal => 1,
    }
}

fn jump_to_next_search(cursor: &mut Cursor, lines: &[BufferLine], direction: &Search) {
    if cursor.horizontal_index == CursorPosition::None {
        return;
    }

    let vertical_index = cursor.vertical_index;
    let mut enumeration: Vec<_> = lines
        .iter()
        .enumerate()
        .filter(|(_, bl)| bl.search_char_position.is_some())
        .collect();

    enumeration.sort_unstable_by(|(current, _), (cmp, _)| {
        sort_by_index(*current, *cmp, vertical_index, direction)
    });

    for (i, line) in enumeration {
        let start = match &line.search_char_position {
            Some(it) => match it.first() {
                Some(s) => s.0,
                None => continue,
            },
            None => continue,
        };

        let downward = direction == &Search::Next;
        if i == vertical_index {
            if let CursorPosition::Absolute { current, .. } = &cursor.horizontal_index {
                if downward && current >= &start || !downward && current <= &start {
                    continue;
                }
            }
        }

        cursor.vertical_index = i;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: start,
            expanded: start,
        };

        break;
    }
}

fn sort_by_index(current: usize, cmp: usize, index: usize, direction: &Search) -> Ordering {
    let downward = direction == &Search::Next;
    if current == cmp {
        return Ordering::Equal;
    }

    if downward {
        if current >= index {
            if current > cmp {
                if cmp >= index {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            } else {
                Ordering::Less
            }
        } else {
            current.cmp(&cmp)
        }
    } else if current <= index {
        if current > cmp {
            Ordering::Less
        } else if cmp <= index {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    } else if current > cmp {
        if cmp <= index {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    } else {
        Ordering::Greater
    }
}

pub fn get_horizontal_index(
    horizontial_index: &CursorPosition,
    line: &BufferLine,
) -> Option<usize> {
    match horizontial_index {
        CursorPosition::Absolute {
            current,
            expanded: _,
        } => Some(*current),
        CursorPosition::End => {
            let index = if line.is_empty() { 0 } else { line.len() - 1 };
            Some(index)
        }
        CursorPosition::None => None,
    }
}

mod test {
    #[test]
    fn sort_by_index_downward() {
        use crate::message::Search;

        let vertical = 5;
        let mut sorted = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        sorted.sort_by(|i, j| super::sort_by_index(*i, *j, vertical, &Search::Next));

        assert_eq!(vec![5, 6, 7, 8, 9, 0, 1, 2, 3, 4], sorted);
    }

    #[test]
    fn sort_by_index_upward() {
        use crate::message::Search;

        let vertical = 5;
        let mut sorted = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        sorted.sort_by(|i, j| super::sort_by_index(*i, *j, vertical, &Search::Previous));

        assert_eq!(vec![5, 4, 3, 2, 1, 0, 9, 8, 7, 6], sorted);
    }
}
