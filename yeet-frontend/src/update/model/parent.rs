use yeet_buffer::{
    message::{Buffer, ViewPortDirection},
    model::{Cursor, CursorPosition},
    update,
};

use crate::model::Model;

pub fn update(model: &mut Model, message: Option<&Buffer>) {
    let buffer = &mut model.parent.buffer;
    let layout = &model.layout.parent;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);

    match &model.parent.path {
        Some(_) => {
            if let Some(message) = message {
                update::update(&model.mode, &model.search, buffer, message);
            }

            let current_filename = match model.current.path.file_name() {
                Some(content) => content.to_str(),
                None => None,
            };

            let current_line = match current_filename {
                Some(content) => buffer.lines.iter().position(|line| line.content == content),
                None => None,
            };

            if let Some(index) = current_line {
                if let Some(cursor) = &mut buffer.cursor {
                    cursor.vertical_index = index;
                } else {
                    buffer.cursor = Some(Cursor {
                        horizontal_index: CursorPosition::None,
                        vertical_index: index,
                        ..Default::default()
                    });
                }

                update::update(
                    &model.mode,
                    &model.search,
                    buffer,
                    &Buffer::MoveViewPort(ViewPortDirection::CenterOnCursor),
                );
            }
        }
        None => {
            buffer.cursor = None;

            update::set_content(&model.mode, buffer, vec![]);

            if let Some(message) = message {
                update::update(&model.mode, &model.search, buffer, message);
            }
        }
    }
}
