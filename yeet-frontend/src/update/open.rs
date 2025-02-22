use yeet_buffer::model::Mode;
use yeet_keymap::message::QuitMode;

use crate::{action::Action, model::Model};

use super::selection::get_current_selected_path;

pub fn open_selected(model: &Model) -> Vec<Action> {
    if model.mode != Mode::Navigation {
        return Vec::new();
    }

    if let Some(selected) = get_current_selected_path(model) {
        if model.settings.selection_to_file_on_open.is_some()
            || model.settings.selection_to_stdout_on_open
        {
            vec![Action::Quit(
                QuitMode::FailOnRunningTasks,
                Some(selected.to_string_lossy().to_string()),
            )]
        } else {
            vec![Action::Open(selected)]
        }
    } else {
        Vec::new()
    }
}
