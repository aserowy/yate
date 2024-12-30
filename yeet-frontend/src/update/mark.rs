use crate::{
    action::Action,
    model::{
        mark::{Marks, MARK_SIGN_ID},
        App, Buffer,
    },
    task::Task,
};

use super::{app, selection, sign};

pub fn add(app: &mut App, marks: &mut Marks, char: char) -> Vec<Action> {
    let buffer = match app::get_focused_mut(app) {
        Buffer::FileTree(it) => it,
        Buffer::_Text(_) => return Vec::new(),
    };

    let selected = selection::get_current_selected_path(buffer);
    if let Some(selected) = selected {
        let removed = marks.entries.insert(char, selected.clone());
        if let Some(removed) = removed {
            sign::unset_sign_for_paths(
                app.buffers.values_mut().collect(),
                vec![removed],
                MARK_SIGN_ID,
            );
        }

        sign::set_sign_for_paths(
            app.buffers.values_mut().collect(),
            vec![selected],
            MARK_SIGN_ID,
        );
    }

    Vec::new()
}

pub fn delete(marks: &mut Marks, buffers: Vec<&mut Buffer>, delete: &Vec<char>) -> Vec<Action> {
    let mut persisted = Vec::new();
    let mut paths = Vec::new();
    for mark in delete {
        let deleted = marks.entries.remove_entry(mark);
        if let Some((mark, path)) = deleted {
            persisted.push(mark);
            paths.push(path);
        }
    }

    sign::unset_sign_for_paths(buffers, paths, MARK_SIGN_ID);

    if persisted.is_empty() {
        Vec::new()
    } else {
        vec![Action::Task(Task::DeleteMarks(persisted))]
    }
}
