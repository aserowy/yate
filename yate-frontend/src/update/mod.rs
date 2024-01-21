use std::path::{Path, PathBuf};

use yate_keymap::action::Action;

use crate::{layout::AppLayout, model::Model};

mod buffer;

pub fn update(model: &mut Model, layout: &AppLayout, message: &Action) {
    match message {
        Action::ChangeKeySequence(sequence) => {
            model.key_sequence = sequence.clone();
        }
        Action::ChangeMode(mode) => {
            model.mode = mode.clone();
        }
        Action::MoveCursor(_) => {
            model.key_sequence = String::new();

            update_current_directory(model, layout, message);
            update_preview(model, layout, message);
        }
        Action::Refresh => {
            update_current_directory(model, layout, message);
            update_parent_directory(model);
            update_preview(model, layout, message);
        }
        Action::SelectCurrent => {
            if let Some(target) = get_target_path(model) {
                if !target.is_dir() {
                    return;
                }

                model.current_path = target;

                update_current_directory(model, layout, message);
                update_parent_directory(model);
                update_preview(model, layout, message);
            }
        }
        Action::SelectParent => {
            if let Some(parent) = &model.current_path.parent() {
                model.current_path = parent.to_path_buf();
            }

            update_current_directory(model, layout, message);
            update_parent_directory(model);
            update_preview(model, layout, message);
        }
        Action::Quit => {}
    }
}

fn update_current_directory(model: &mut Model, layout: &AppLayout, message: &Action) {
    let path = Path::new(&model.current_path);

    model.current_directory.view_port.height = usize::from(layout.current_directory.height);
    model.current_directory.view_port.width = usize::from(layout.current_directory.width);

    let mut content: Vec<_> = std::fs::read_dir(path)
        .unwrap()
        .map(|entry| {
            entry
                .unwrap()
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect();

    content.sort_unstable();

    model.current_directory.lines = content;

    buffer::update(&mut model.current_directory, message);
}

fn update_parent_directory(model: &mut Model) {
    let path = Path::new(&model.current_path);

    match path.parent() {
        Some(parent) => {
            let mut content: Vec<_> = std::fs::read_dir(parent)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();

            content.sort_unstable();

            model.parent_directory.paths = content;
        }
        None => model.parent_directory.paths = vec![],
    }
}

fn update_preview(model: &mut Model, layout: &AppLayout, message: &Action) {
    if let Some(target) = get_target_path(model) {
        model.preview.view_port.height = usize::from(layout.current_directory.height);
        model.preview.view_port.width = usize::from(layout.current_directory.width);

        let content = if target.is_dir() {
            let mut content: Vec<_> = std::fs::read_dir(target)
                .unwrap()
                .map(|entry| {
                    entry
                        .unwrap()
                        .path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string()
                })
                .collect();

            content.sort_unstable();

            content
        } else {
            Vec::new()
        };

        model.preview.lines = content;

        buffer::update(&mut model.preview, message);
    }
}

fn get_target_path(model: &Model) -> Option<PathBuf> {
    let buffer = &model.current_directory;
    if let Some(cursor) = &buffer.cursor {
        let current = &buffer.lines[cursor.vertical_index];
        let target = model.current_path.join(current);

        if target.exists() {
            Some(target)
        } else {
            None
        }
    } else {
        None
    }
}
