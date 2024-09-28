use yeet_buffer::model::Buffer;

use crate::{
    action::Action,
    model::{Model, PreviewContent},
};

pub fn search_in_buffers(model: &mut Model, search: Option<String>) {
    let search = match search {
        Some(it) => it,
        None => {
            clear_search(model);
            return;
        }
    };

    if model.files.parent.path.is_some() {
        set_search_char_positions(&mut model.files.parent.buffer, search.as_str());
    }

    set_search_char_positions(&mut model.files.current.buffer, search.as_str());

    let preview = match &mut model.files.preview {
        PreviewContent::Buffer(dir) => dir,
        _ => return,
    };

    if preview.path.is_dir() {
        set_search_char_positions(&mut preview.buffer, search.as_str());
    }
}

pub fn clear_search(model: &mut Model) -> Vec<Action> {
    for line in &mut model.files.parent.buffer.lines {
        line.search_char_position = None;
    }
    for line in &mut model.files.current.buffer.lines {
        line.search_char_position = None;
    }
    if let PreviewContent::Buffer(dir) = &mut model.files.preview {
        for line in &mut dir.buffer.lines {
            line.search_char_position = None;
        }
    }
    Vec::new()
}

fn set_search_char_positions(buffer: &mut Buffer, search: &str) {
    let smart_case = search.chars().all(|c| c.is_ascii_lowercase());
    let search_length = search.chars().count();

    for line in &mut buffer.lines {
        line.search_char_position = None;

        let mut content = line.content.to_stripped_string();
        let lower = content.to_lowercase();
        if smart_case {
            content = lower;
        };

        let start = match content.find(search) {
            Some(it) => content[..it].chars().count(),
            None => continue,
        };

        line.search_char_position = Some(vec![(start, search_length)]);
    }
}
