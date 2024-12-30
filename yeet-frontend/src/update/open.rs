use yeet_buffer::model::Mode;
use yeet_keymap::message::QuitMode;

use crate::{action::Action, model::FileTreeBuffer, settings::Settings};

use super::selection::get_current_selected_path;

pub fn selected(settings: &Settings, mode: &Mode, buffer: &mut FileTreeBuffer) -> Vec<Action> {
    if mode != &Mode::Navigation {
        return Vec::new();
    }

    if let Some(selected) = get_current_selected_path(buffer) {
        if settings.selection_to_file_on_open.is_some() || settings.selection_to_stdout_on_open {
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
