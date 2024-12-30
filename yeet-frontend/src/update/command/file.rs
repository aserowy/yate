use std::path::{Path, PathBuf};

use yeet_keymap::message::KeymapMessage;

use crate::{
    action::{self, Action},
    event::Message,
    model::{mark::Marks, FileTreeBuffer, FileTreeBufferSectionBuffer},
    task::Task,
};

pub fn copy_selection(
    marks: &Marks,
    buffer: &FileTreeBufferSectionBuffer,
    target: &str,
) -> Vec<Action> {
    let mut actions = Vec::new();
    if let Some(path) = &buffer.resolve_path() {
        tracing::info!("copying path: {:?}", path);
        match get_target_file_path(marks, target, path) {
            Ok(target) => actions.push(Action::Task(Task::CopyPath(path.to_path_buf(), target))),
            Err(err) => {
                actions.push(Action::EmitMessages(vec![Message::Error(err)]));
            }
        };
    }
    actions
}

pub fn delete_selection(buffer: &FileTreeBufferSectionBuffer) -> Vec<Action> {
    let mut actions = Vec::new();
    if let Some(path) = &buffer.resolve_path() {
        tracing::info!("deleting path: {:?}", path);
        actions.push(Action::Task(Task::DeletePath(path.to_path_buf())));
    } else {
        tracing::warn!("deleting path failed: no path in preview set");
    }

    actions
}

pub fn rename_selection(
    marks: &Marks,
    buffer: &FileTreeBufferSectionBuffer,
    target: &str,
) -> Vec<Action> {
    let mut actions = Vec::new();
    if let Some(path) = &buffer.resolve_path() {
        tracing::info!("renaming path: {:?}", path);
        match get_target_file_path(marks, target, path) {
            Ok(target) => {
                actions.push(Action::Task(Task::RenamePath(path.to_path_buf(), target)));
            }
            Err(err) => {
                actions.push(Action::EmitMessages(vec![Message::Error(err)]));
            }
        };
    }

    actions
}

pub fn refresh(buffer: &FileTreeBuffer) -> Vec<Action> {
    let navigation = if let Some(path) = &buffer.preview.resolve_path() {
        KeymapMessage::NavigateToPathAsPreview(path.to_path_buf())
    } else {
        KeymapMessage::NavigateToPath(buffer.current.path.clone())
    };

    vec![action::emit_keymap(navigation)]
}

fn get_target_file_path(marks: &Marks, target: &str, path: &Path) -> Result<PathBuf, String> {
    let file_name = match path.file_name() {
        Some(it) => it,
        None => return Err(format!("could not resolve file name from path {:?}", path)),
    };

    let target = if target.starts_with('\'') {
        let mark = match target.chars().nth(1) {
            Some(it) => it,
            None => return Err("invalid mark format".to_string()),
        };

        if let Some(path) = marks.entries.get(&mark) {
            path.to_path_buf()
        } else {
            return Err(format!("mark '{}' not found", mark));
        }
    } else if path.is_relative() {
        let current = match path.parent() {
            Some(it) => it,
            None => return Err(format!("could not resolve parent from path {:?}", path)),
        };

        let path = Path::new(path);
        current.join(path)
    } else {
        PathBuf::from(path)
    };

    let target_file = target.join(file_name);
    if target.is_dir() && target.exists() && !target_file.exists() {
        Ok(target.join(file_name))
    } else {
        Err("target path is not valid".to_string())
    }
}
