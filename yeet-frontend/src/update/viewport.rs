use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::Mode,
};

use crate::{
    action::Action,
    model::{history::History, App, FileTreeBufferSection},
};

use super::{history, selection};

pub fn relocate(
    app: &mut App,
    history: &History,
    mode: &Mode,
    direction: &ViewPortDirection,
) -> Vec<Action> {
    let msg = BufferMessage::MoveViewPort(direction.clone());

    yeet_buffer::update(
        &mut buffer.current_vp,
        &mut buffer.current_cursor,
        mode,
        &mut buffer.current.buffer,
        &msg,
    );

    let mut actions = Vec::new();
    if let Some(path) = selection::get_current_selected_path(buffer) {
        let selection = history::get_selection_from_history(history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(
            FileTreeBufferSection::Preview,
            path,
            selection,
        ));
    }

    actions
}
