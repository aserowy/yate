use std::path::Path;

use yeet_buffer::{
    message::{BufferMessage, CursorDirection, Search},
    model::{viewport::ViewPort, BufferResult, Cursor, Mode, SearchDirection, TextBuffer},
};

use crate::{
    action::Action,
    model::{
        history::History, App, Buffer, FileTreeBufferSection, FileTreeBufferSectionBuffer, State,
    },
};

use super::{
    app,
    history::get_selection_from_history,
    register::{get_direction_from_search_register, get_register},
    search, selection,
};

pub fn set_cursor_index_to_selection(
    viewport: &mut Option<ViewPort>,
    cursor: &mut Option<Cursor>,
    mode: &Mode,
    text_buffer: &mut TextBuffer,
    selection: &str,
) -> bool {
    let result = yeet_buffer::update(
        viewport,
        cursor,
        mode,
        text_buffer,
        vec![&BufferMessage::SetCursorToLineContent(
            selection.to_string(),
        )],
    );

    result.contains(&BufferResult::CursorPositionChanged)
}

pub fn set_cursor_index_with_history(
    history: &History,
    viewport: &mut Option<ViewPort>,
    cursor: &mut Option<Cursor>,
    mode: &Mode,
    buffer: &mut TextBuffer,
    path: &Path,
) -> bool {
    if let Some(history) = get_selection_from_history(history, path) {
        set_cursor_index_to_selection(viewport, cursor, mode, buffer, history)
    } else {
        false
    }
}

pub fn relocate(
    app: &mut App,
    state: &mut State,
    rpt: &usize,
    mtn: &CursorDirection,
) -> Vec<Action> {
    if matches!(*mtn, CursorDirection::Search(_)) {
        let term = get_register(&state.register, &'/');
        search::search_in_buffers(app.buffers.values_mut().collect(), term);
    }

    let buffer = match app::get_focused_mut(app) {
        Buffer::FileTree(it) => it,
        Buffer::_Text(_) => todo!(),
    };

    let premotion_preview_path = match &buffer.preview {
        FileTreeBufferSectionBuffer::Image(path, _)
        | FileTreeBufferSectionBuffer::Text(path, _) => Some(path.clone()),
        FileTreeBufferSectionBuffer::None => None,
    };

    let msg = BufferMessage::MoveCursor(*rpt, mtn.clone());
    if let CursorDirection::Search(drctn) = mtn {
        let current_drctn = match get_direction_from_search_register(&state.register) {
            Some(it) => it,
            None => return Vec::new(),
        };

        let dr = match (drctn, current_drctn) {
            (Search::Next, SearchDirection::Down) => Search::Next,
            (Search::Next, SearchDirection::Up) => Search::Previous,
            (Search::Previous, SearchDirection::Down) => Search::Previous,
            (Search::Previous, SearchDirection::Up) => Search::Next,
        };

        let msg = BufferMessage::MoveCursor(*rpt, CursorDirection::Search(dr.clone()));
        yeet_buffer::update(
            &mut buffer.current_vp,
            &mut buffer.current_cursor,
            &state.modes.current,
            &mut buffer.current.buffer,
            &msg,
        );
    } else {
        yeet_buffer::update(
            &mut buffer.current_vp,
            &mut buffer.current_cursor,
            &state.modes.current,
            &mut buffer.current.buffer,
            &msg,
        );
    };

    let mut actions = Vec::new();
    let current_preview_path = selection::get_current_selected_path(&buffer);
    if premotion_preview_path == current_preview_path {
        return actions;
    }

    if let Some(path) = selection::get_current_selected_path(&buffer) {
        let selection = get_selection_from_history(&state.history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(
            FileTreeBufferSection::Preview,
            path,
            selection,
        ));
    }

    actions
}
