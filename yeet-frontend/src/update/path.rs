use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use yeet_buffer::{
    message::BufferMessage,
    model::{ansi::Ansi, BufferLine, Cursor, Mode, TextBuffer},
};

use crate::{
    action::Action,
    model::{
        history::History, junkyard::JunkYard, mark::Marks, qfix::QuickFix, Buffer,
        FileTreeBufferSection, FileTreeBufferSectionBuffer,
    },
};

use super::{
    history::get_selection_from_history,
    junkyard::remove_from_junkyard,
    selection,
    sign::{set_sign_if_marked, set_sign_if_qfix},
};

#[tracing::instrument(skip(buffers))]
pub fn add(
    history: &History,
    marks: &Marks,
    qfix: &QuickFix,
    mode: &Mode,
    buffers: Vec<&mut Buffer>,
    paths: &[PathBuf],
) -> Vec<Action> { 
    let mut actions = Vec::new();
    for buffer in buffers {
        let buffer = match buffer {
            Buffer::FileTree(it) => it,
            _ => continue,
        };

        let mut buffer_contents = vec![(
            buffer.current.path.as_path(),
            &mut buffer.current_vp,
            &mut buffer.current_cursor,
            &mut buffer.current.buffer,
            mode == &Mode::Navigation,
        )];

        if let FileTreeBufferSectionBuffer::Text(path, text_buffer) = &mut buffer.parent {
            buffer_contents.push((
                path.as_path(),
                &mut buffer.parent_vp,
                &mut buffer.parent_cursor,
                text_buffer,
                path.is_dir(),
            ));
        }

        if let FileTreeBufferSectionBuffer::Text(path, text_buffer) = &mut buffer.preview {
            buffer_contents.push((
                path.as_path(),
                &mut buffer.preview_vp,
                &mut buffer.preview_cursor,
                text_buffer,
                path.is_dir(),
            ));
        }

        for (path, viewport, cursor, buffer, sort) in buffer_contents {
            let paths_for_buffer: Vec<_> =
                paths.iter().filter(|p| p.parent() == Some(path)).collect();
            if paths_for_buffer.is_empty() {
                continue;
            }

            let mut selection = match cursor {
                Some(it) => get_selected_content_from_buffer(it, buffer),
                None => None,
            };

            let indexes = buffer
                .lines
                .iter()
                .enumerate()
                .map(|(i, bl)| {
                    let content = bl.content.to_stripped_string();
                    let key = if content.contains('/') {
                        content.split('/').collect::<Vec<_>>()[0].to_string()
                    } else {
                        content.clone()
                    };

                    (key, i)
                })
                .collect::<HashMap<_, _>>();

            for path in paths_for_buffer {
                if let Some(basename) = path.file_name().and_then(|oss| oss.to_str()) {
                    let mut line = from(path);
                    set_sign_if_marked(marks, &mut line, path);
                    set_sign_if_qfix(qfix, &mut line, path);

                    if let Some(index) = indexes.get(basename) {
                        buffer.lines[*index] = line;
                    } else {
                        buffer.lines.push(line);
                    }

                    selection = selection.map(|sl| {
                        if sl.starts_with(&[basename, "/"].concat()) {
                            basename.to_owned()
                        } else {
                            sl
                        }
                    });
                }
            }

            if sort {
                yeet_buffer::update(
                    viewport,
                    cursor,
                    mode,
                    buffer,
                    vec![&BufferMessage::SortContent(super::SORT)],
                );
            }

            if let Some(selection) = selection {
                yeet_buffer::update(
                    viewport,
                    cursor,
                    mode,
                    buffer,
                    vec![&BufferMessage::SetCursorToLineContent(selection)],
                );
            }
        }

        if let Some(path) = selection::get_current_selected_path(buffer) {
            let selection = get_selection_from_history(history, &path).map(|s| s.to_owned());
            actions.push(Action::Load(
                FileTreeBufferSection::Preview,
                path,
                selection,
            ));
        }
    }

    actions
}

fn get_selected_content_from_buffer(cursor: &Cursor, model: &TextBuffer) -> Option<String> {
    model
        .lines
        .get(cursor.vertical_index)
        .map(|line| line.content.to_stripped_string())
}

fn from(path: &Path) -> BufferLine {
    let content = match path.file_name() {
        Some(content) => content.to_str().unwrap_or("").to_string(),
        None => "".to_string(),
    };

    let content = if path.is_dir() {
        format!("\x1b[94m{}\x1b[39m", content)
    } else {
        content
    };

    BufferLine {
        content: Ansi::new(&content),
        ..Default::default()
    }
}

#[tracing::instrument(skip(junk, buffers))]
pub fn remove(
    history: &History,
    junk: &mut JunkYard,
    mode: &Mode,
    buffers: Vec<&mut Buffer>,
    path: &Path,
) -> Vec<Action> {
    let mut actions = Vec::new();
    for buffer in buffers {
        let buffer = match buffer {
            Buffer::FileTree(it) => it,
            _ => continue,
        };

        if path.starts_with(junk.path.clone()) {
            remove_from_junkyard(junk, path);
        }

        let current_selection = match &buffer.current_cursor {
            Some(it) => get_selected_content_from_buffer(it, &buffer.current.buffer),
            None => None,
        };

        let mut buffer_contents = vec![(
            buffer.current.path.as_path(),
            &mut buffer.current_vp,
            &mut buffer.current_cursor,
            &mut buffer.current.buffer,
        )];

        if let FileTreeBufferSectionBuffer::Text(path, text_buffer) = &mut buffer.parent {
            buffer_contents.push((
                path.as_path(),
                &mut buffer.parent_vp,
                &mut buffer.parent_cursor,
                text_buffer,
            ));
        }

        if let FileTreeBufferSectionBuffer::Text(path, text_buffer) = &mut buffer.preview {
            buffer_contents.push((
                path.as_path(),
                &mut buffer.preview_vp,
                &mut buffer.preview_cursor,
                text_buffer,
            ));
        }

        if let Some(parent) = path.parent() {
            if let Some((_, viewport, cursor, buffer)) = buffer_contents
                .into_iter()
                .find(|(p, _, _, _)| p == &parent)
            {
                if let Some(basename) = path.file_name().and_then(|oss| oss.to_str()) {
                    let index = buffer
                        .lines
                        .iter()
                        .enumerate()
                        .find(|(_, bl)| bl.content.to_stripped_string() == basename)
                        .map(|(i, _)| i);

                    if let Some(index) = index {
                        yeet_buffer::update(
                            viewport,
                            cursor,
                            mode,
                            buffer,
                            vec![&BufferMessage::RemoveLine(index)],
                        );
                    }
                }
            }
        }

        if let Some(selection) = current_selection {
            yeet_buffer::update(
                &mut buffer.current_vp,
                &mut buffer.current_cursor,
                mode,
                &mut buffer.current.buffer,
                vec![&BufferMessage::SetCursorToLineContent(selection)],
            );
        };

        if let Some(path) = selection::get_current_selected_path(buffer) {
            let selection = get_selection_from_history(history, &path).map(|s| s.to_owned());
            actions.push(Action::Load(
                FileTreeBufferSection::Preview,
                path,
                selection,
            ));
        }
    }

    actions
}
