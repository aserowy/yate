use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use yeet_keymap::message::Mode;

use crate::model::Model;

use super::{bufferline, mark, preview, qfix};

pub fn add(model: &mut Model, paths: &[PathBuf]) {
    add_paths(model, paths);

    preview::selected_path(model);
    preview::viewport(model);
}

fn add_paths(model: &mut Model, paths: &[PathBuf]) {
    let mut buffer = vec![(
        model.current.path.as_path(),
        &mut model.current.buffer,
        model.mode == Mode::Navigation,
    )];

    if let Some(preview) = &model.preview.path {
        buffer.push((preview, &mut model.preview.buffer, true));
    }

    if let Some(parent) = &model.parent.path {
        buffer.push((parent, &mut model.parent.buffer, true));
    }

    for (path, buffer, sort) in buffer {
        let paths_for_buffer: Vec<_> = paths.iter().filter(|p| p.parent() == Some(path)).collect();
        let indexes = buffer
            .lines
            .iter()
            .enumerate()
            .map(|(i, bl)| {
                let key = if bl.content.contains('/') {
                    bl.content.split('/').collect::<Vec<_>>()[0].to_string()
                } else {
                    bl.content.clone()
                };

                (key, i)
            })
            .collect::<HashMap<_, _>>();

        for path in paths_for_buffer {
            if let Some(basename) = path.file_name().and_then(|oss| oss.to_str()) {
                let mut line = bufferline::from(path);
                mark::set_sign_if_marked(&model.marks, &mut line, path);
                qfix::set_sign_if_qfix(&model.qfix, &mut line, path);

                if let Some(index) = indexes.get(basename) {
                    buffer.lines[*index] = line;
                } else {
                    buffer.lines.push(line);
                }
            }
        }

        if sort {
            super::sort_content(&model.mode, buffer);
        }

        super::buffer::cursor::validate(&model.mode, buffer);

        // TODO: correct cursor to stay on selection
    }
}

pub fn remove(model: &mut Model, path: &Path) {
    remove_path(model, path);

    preview::selected_path(model);
    preview::viewport(model);
}

fn remove_path(model: &mut Model, path: &Path) {
    let mut buffer = vec![(model.current.path.as_path(), &mut model.current.buffer)];

    if let Some(preview) = &model.preview.path {
        buffer.push((preview, &mut model.preview.buffer));
    }

    if let Some(parent) = &model.parent.path {
        buffer.push((parent, &mut model.parent.buffer));
    }

    if let Some(parent) = path.parent() {
        if let Some((_, buffer)) = buffer.into_iter().find(|(p, _)| p == &parent) {
            if let Some(basename) = path.file_name().and_then(|oss| oss.to_str()) {
                let index = buffer
                    .lines
                    .iter()
                    .enumerate()
                    .find(|(_, bl)| bl.content == basename)
                    .map(|(i, _)| i);

                if let Some(index) = index {
                    buffer.lines.remove(index);
                    super::buffer::cursor::validate(&model.mode, buffer);
                }
            }
        }
    }
}
