use crate::{
    message::{BufferMessage, CursorDirection},
    model::{viewport::ViewPort, BufferResult, Cursor, CursorPosition, Mode, TextBuffer},
};

mod cursor;
mod find;
mod modification;
pub mod viewport;
mod word;

pub fn update(
    viewport: Option<&ViewPort>,
    cursor: Option<&Cursor>,
    mode: &Mode,
    buffer: &mut TextBuffer,
    messages: Vec<&BufferMessage>,
) -> (Option<ViewPort>, Option<Cursor>, Vec<BufferResult>) {
    let mut actions = Vec::new();

    let mut viewport = match viewport {
        Some(it) => Some(it.clone()),
        None => None,
    };

    let mut cursor = match cursor {
        Some(it) => Some(it.clone()),
        None => None,
    };

    for message in messages.iter() {
        actions.push(update_buffer(
            &mut viewport,
            &mut cursor,
            mode,
            buffer,
            message,
        ));
    }

    (viewport, cursor, actions.into_iter().flatten().collect())
}

fn update_buffer(
    viewport: &mut Option<ViewPort>,
    cursor: &mut Option<Cursor>,
    mode: &Mode,
    buffer: &mut TextBuffer,
    message: &BufferMessage,
) -> Vec<BufferResult> {
    tracing::debug!("handling buffer message: {:?}", message);

    let result = match message {
        // TODO: repeat actions by count when switching from insert to normal
        // count is entered before going into insert. ChangeMode with count? Or Insert with count?
        BufferMessage::ChangeMode(from, to) => {
            if from == &Mode::Insert && to != &Mode::Insert {
                buffer.undo.close_transaction();

                if let Some(cursor) = cursor {
                    cursor::update_by_direction(cursor, mode, buffer, &1, &CursorDirection::Left);
                    if let Some(viewport) = viewport {
                        viewport::update_by_cursor(viewport, cursor, buffer);
                    }
                }
            }
            Vec::new()
        }
        BufferMessage::Modification(count, modification) => {
            if let Some(cursor) = cursor {
                let changes = modification::update(cursor, mode, buffer, count, modification);
                if let Some(viewport) = viewport {
                    viewport::update_by_cursor(viewport, cursor, buffer);
                }

                if let Some(changes) = changes {
                    buffer.undo.add(mode, changes);
                }
            }
            Vec::new()
        }
        BufferMessage::MoveCursor(count, direction) => {
            if let Some(cursor) = cursor {
                let result = cursor::update_by_direction(cursor, mode, buffer, count, direction);
                if let Some(viewport) = viewport {
                    viewport::update_by_cursor(viewport, cursor, buffer);
                }

                result
            } else {
                Vec::new()
            }
            // TODO: history::add_history_entry(&mut model.history, selected.as_path());
        }
        BufferMessage::MoveViewPort(direction) => {
            let viewport = match viewport {
                Some(it) => it,
                None => return Vec::new(),
            };

            viewport::update_by_direction(viewport, cursor, buffer, direction);
            if let Some(cursor) = cursor {
                viewport::update_by_cursor(viewport, cursor, buffer);
            }

            Vec::new()
        }
        BufferMessage::RemoveLine(index) => {
            buffer.lines.remove(*index);

            if let Some(cursor) = cursor {
                cursor::set_to_inbound_position(cursor, mode, buffer);
                if let Some(viewport) = viewport {
                    viewport::update_by_cursor(viewport, cursor, buffer);
                }
            }

            Vec::new()
        }
        BufferMessage::ResetCursor => {
            if let Some(cursor) = cursor {
                cursor.vertical_index = 0;

                cursor.horizontal_index = match &cursor.horizontal_index {
                    CursorPosition::Absolute {
                        current: _,
                        expanded: _,
                    } => CursorPosition::Absolute {
                        current: 0,
                        expanded: 0,
                    },
                    CursorPosition::End => CursorPosition::End,
                    CursorPosition::None => CursorPosition::None,
                };
                if let Some(viewport) = viewport {
                    viewport.horizontal_index = 0;
                    viewport.vertical_index = 0;

                    viewport::update_by_cursor(viewport, cursor, buffer);
                }
            }

            Vec::new()
        }
        BufferMessage::SaveBuffer => {
            let changes = buffer.undo.save();
            vec![BufferResult::Changes(changes)]
        }
        BufferMessage::SetContent(content) => {
            // TODO: optional selection?
            buffer.lines = content.to_vec();

            if let Some(cursor) = cursor {
                cursor::set_to_inbound_position(cursor, mode, buffer);
                if let Some(viewport) = viewport {
                    viewport::update_by_cursor(viewport, cursor, buffer);
                }
            }

            Vec::new()
        }
        BufferMessage::SetCursorToLineContent(content) => {
            let cursor = match cursor {
                Some(it) => it,
                None => return Vec::new(),
            };

            let line = buffer
                .lines
                .iter()
                .enumerate()
                .find(|(_, line)| &line.content.to_stripped_string() == content);

            if let Some((index, _)) = line {
                cursor.vertical_index = index;
                cursor.hide_cursor_line = false;

                cursor::set_to_inbound_position(cursor, mode, buffer);
                if let Some(viewport) = viewport {
                    viewport::update_by_cursor(viewport, cursor, buffer);
                }

                vec![BufferResult::CursorPositionChanged]
            } else {
                Vec::new()
            }
        }
        BufferMessage::SortContent(sort) => {
            // TODO: cursor should stay on current selection
            buffer.lines.sort_unstable_by(sort);
            if let Some(cursor) = cursor {
                cursor::set_to_inbound_position(cursor, mode, buffer);
                if let Some(viewport) = viewport {
                    viewport::update_by_cursor(viewport, cursor, buffer);
                }
            }
            Vec::new()
        }
        BufferMessage::UpdateViewPortByCursor => Vec::new(),
    };

    result
}
