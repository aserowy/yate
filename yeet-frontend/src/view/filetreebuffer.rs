use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use ratatui_image::Image;
use yeet_buffer::{
    model::{viewport::ViewPort, Cursor, Mode},
    view,
};

use crate::model::{FileTreeBuffer, FileTreeBufferSectionBuffer};

pub fn view(
    mode: &Mode,
    viewport: &ViewPort,
    cursor: &Cursor,
    buffer: &FileTreeBuffer,
    frame: &mut Frame,
    horizontal_offset: u16,
    vertical_offset: u16,
) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_ratios([(1, 5), (2, 5), (2, 5)]))
        .split(Rect {
            x: horizontal_offset,
            y: vertical_offset,
            width: viewport.width,
            height: viewport.height,
        });

    render_buffer(
        &buffer.parent_cursor,
        mode,
        frame,
        &buffer.parent,
        layout[0].x,
        layout[0].width,
        layout[0].y,
        layout[0].height,
    );

    let mut viewport = viewport.clone();
    viewport.height = layout[1].height;
    viewport.width = layout[1].width;

    view::view(
        &viewport,
        Some(cursor),
        mode,
        &buffer.current.buffer,
        frame,
        layout[1].x,
        layout[1].y,
    );

    render_buffer(
        &buffer.preview_cursor,
        mode,
        frame,
        &buffer.preview,
        layout[2].x,
        layout[2].width,
        layout[2].y,
        layout[2].height,
    );
}

fn render_buffer(
    cursor: &Option<Cursor>,
    mode: &Mode,
    frame: &mut Frame,
    buffer_type: &FileTreeBufferSectionBuffer,
    horizontal_offset: u16,
    width: u16,
    vertical_offset: u16,
    height: u16,
) {
    let mut viewport = ViewPort::default();
    viewport.height = height;
    viewport.width = width;

    match buffer_type {
        FileTreeBufferSectionBuffer::Text(_, buffer) => {
            if let Some(cursor) = cursor {
                yeet_buffer::update::viewport::update_by_cursor(&mut viewport, cursor, &buffer);
            };

            view::view(
                &viewport,
                cursor.as_ref(),
                mode,
                buffer,
                frame,
                horizontal_offset,
                vertical_offset,
            );
        }
        FileTreeBufferSectionBuffer::Image(_, protocol) => {
            let rect = Rect {
                x: horizontal_offset,
                y: vertical_offset,
                width: viewport.width,
                height: viewport.height,
            };

            frame.render_widget(Image::new(protocol), rect);
        }
        FileTreeBufferSectionBuffer::None => {
            view::view(
                &viewport,
                None,
                mode,
                &Default::default(),
                frame,
                horizontal_offset,
                vertical_offset,
            );
        }
    };
}
