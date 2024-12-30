use std::{mem, path::Path};

use yeet_buffer::{
    message::ViewPortDirection,
    model::{Cursor, CursorPosition, TextBuffer},
    update::viewport,
};

use crate::{
    action::Action,
    model::{
        history::History, mark::Marks, App, Buffer, FileTreeBuffer, FileTreeBufferSection,
        FileTreeBufferSectionBuffer,
    },
};

use super::{app, history, selection};

#[tracing::instrument(skip(app))]
pub fn mark(app: &mut App, history: &History, marks: &Marks, char: &char) -> Vec<Action> {
    let buffer = match app::get_focused_mut(app) {
        (_, _, Buffer::FileTree(it)) => it,
        (_, _, Buffer::_Text(_)) => todo!(),
    };

    let path = match marks.entries.get(char) {
        Some(it) => it.clone(),
        None => return Vec::new(),
    };

    let selection = path
        .file_name()
        .map(|oss| oss.to_string_lossy().to_string());

    let path = match path.parent() {
        Some(parent) => parent,
        None => &path,
    };

    navigate_to_path_with_selection(history, buffer, path, &selection)
}

#[tracing::instrument(skip(app, history))]
pub fn path(app: &mut App, history: &History, path: &Path) -> Vec<Action> {
    let buffer = match app::get_focused_mut(app) {
        (_, _, Buffer::FileTree(it)) => it,
        (_, _, Buffer::_Text(_)) => todo!(),
    };

    let (path, selection) = if path.is_file() {
        tracing::info!("path is a file, not a directory: {:?}", path);

        let selection = path
            .file_name()
            .map(|oss| oss.to_string_lossy().to_string());

        match path.parent() {
            Some(parent) => (parent, selection),
            None => {
                tracing::warn!(
                    "parent from path with file name could not get resolved: {:?}",
                    path
                );
                return Vec::new();
            }
        }
    } else {
        (path, None)
    };

    navigate_to_path_with_selection(history, buffer, path, &selection)
}

pub fn path_as_preview(app: &mut App, history: &History, path: &Path) -> Vec<Action> {
    let buffer = match app::get_focused_mut(app) {
        (_, _, Buffer::FileTree(it)) => it,
        (_, _, Buffer::_Text(_)) => todo!(),
    };

    let selection = path
        .file_name()
        .map(|oss| oss.to_string_lossy().to_string());

    let path = match path.parent() {
        Some(parent) => parent,
        None => path,
    };

    navigate_to_path_with_selection(history, buffer, path, &selection)
}

#[tracing::instrument(skip(buffer, history))]
pub fn navigate_to_path_with_selection(
    history: &History,
    buffer: &mut FileTreeBuffer,
    path: &Path,
    selection: &Option<String>,
) -> Vec<Action> {
    if path.is_file() {
        tracing::warn!("path is a file, not a directory: {:?}", path);
        return Vec::new();
    }

    if !path.exists() {
        tracing::warn!("path does not exist: {:?}", path);
        return Vec::new();
    }

    let selection = match selection {
        Some(it) => Some(it.to_owned()),
        None => {
            tracing::trace!("getting selection from history for path: {:?}", path);
            history::get_selection_from_history(history, path).map(|history| history.to_owned())
        }
    };

    tracing::trace!("resolved selection: {:?}", selection);

    buffer.preview = FileTreeBufferSectionBuffer::None;

    let mut actions = Vec::new();
    actions.push(Action::Load(
        FileTreeBufferSection::Current,
        path.to_path_buf(),
        selection.clone(),
    ));

    let parent = path.parent();
    if let Some(parent) = parent {
        if parent != path {
            actions.push(Action::Load(
                FileTreeBufferSection::Parent,
                parent.to_path_buf(),
                path.file_name().map(|it| it.to_string_lossy().to_string()),
            ));
        }
    }

    actions
}

#[tracing::instrument(skip(app))]
pub fn parent(app: &mut App) -> Vec<Action> {
    let (vp, cursor, buffer) = match app::get_focused_mut(app) {
        (vp, cursor, Buffer::FileTree(it)) => (vp, cursor, it),
        (_vp, _cursor, Buffer::_Text(_)) => todo!(),
    };

    if let Some(path) = buffer.current.path.clone().parent() {
        if buffer.current.path == path {
            return Vec::new();
        }

        let mut actions = Vec::new();

        if let Some(parent) = path.parent() {
            tracing::trace!("loading parent: {:?}", parent);

            actions.push(Action::Load(
                FileTreeBufferSection::Parent,
                parent.to_path_buf(),
                path.file_name().map(|it| it.to_string_lossy().to_string()),
            ));
        }

        let parent_buffer =
            match mem::replace(&mut buffer.parent, FileTreeBufferSectionBuffer::None) {
                FileTreeBufferSectionBuffer::Text(_, buffer) => buffer,
                FileTreeBufferSectionBuffer::Image(_, _) | FileTreeBufferSectionBuffer::None => {
                    TextBuffer::default()
                }
            };

        let current_path = mem::replace(&mut buffer.current.path, path.to_path_buf());
        let current_buffer = mem::replace(&mut buffer.current.buffer, parent_buffer);

        buffer.preview = FileTreeBufferSectionBuffer::Text(current_path, current_buffer);
        buffer.preview_cursor = Some(Default::default());

        if let Some(parent_cursor) = &mut buffer.parent_cursor {
            mem_swap_cursor(cursor, parent_cursor);

            if let Some(preview_cursor) = &mut buffer.preview_cursor {
                mem_swap_cursor(parent_cursor, preview_cursor);
            }
        }

        viewport::update_by_direction(
            vp,
            Some(cursor),
            &buffer.current.buffer,
            &ViewPortDirection::CenterOnCursor,
        );

        actions
    } else {
        Vec::new()
    }
}

#[tracing::instrument(skip(app, history))]
pub fn selected(app: &mut App, history: &mut History) -> Vec<Action> {
    let (vp, cursor, buffer) = match app::get_focused_mut(app) {
        (vp, cursor, Buffer::FileTree(it)) => (vp, cursor, it),
        (_vp, _cursor, Buffer::_Text(_)) => todo!(),
    };

    if let Some(selected) = selection::get_current_selected_path(buffer) {
        if buffer.current.path == selected || !selected.is_dir() {
            return Vec::new();
        }

        history::add_history_entry(history, selected.as_path());

        let mut actions = Vec::new();
        let preview_buffer =
            match mem::replace(&mut buffer.preview, FileTreeBufferSectionBuffer::None) {
                FileTreeBufferSectionBuffer::Text(_, buffer) => buffer,
                FileTreeBufferSectionBuffer::Image(_, _) | FileTreeBufferSectionBuffer::None => {
                    let history = history::get_selection_from_history(history, selected.as_path())
                        .map(|s| s.to_string());

                    actions.push(Action::Load(
                        FileTreeBufferSection::Current,
                        selected.to_path_buf(),
                        history,
                    ));

                    TextBuffer::default()
                }
            };

        buffer.parent = FileTreeBufferSectionBuffer::Text(
            mem::replace(&mut buffer.current.path, selected.to_path_buf()),
            mem::replace(&mut buffer.current.buffer, preview_buffer),
        );

        if let Some(parent_cursor) = &mut buffer.parent_cursor {
            mem_swap_cursor(cursor, parent_cursor);
        }

        if let Some(preview_cursor) = &mut buffer.preview_cursor {
            mem_swap_cursor(cursor, preview_cursor);
        } else {
            cursor.horizontal_index = CursorPosition::Absolute {
                current: 0,
                expanded: 0,
            };

            if let Some(selected) = selection::get_current_selected_path(buffer) {
                tracing::trace!("loading selection: {:?}", selected);

                let history = history::get_selection_from_history(history, selected.as_path())
                    .map(|s| s.to_string());

                actions.push(Action::Load(
                    FileTreeBufferSection::Preview,
                    selected.to_path_buf(),
                    history,
                ));
            }
        }

        viewport::update_by_direction(
            vp,
            Some(cursor),
            &buffer.current.buffer,
            &ViewPortDirection::CenterOnCursor,
        );

        actions
    } else {
        Vec::new()
    }
}

fn mem_swap_cursor(dest_cursor: &mut Cursor, src_cursor: &mut Cursor) {
    mem::swap(
        &mut dest_cursor.vertical_index,
        &mut src_cursor.vertical_index,
    );
}
