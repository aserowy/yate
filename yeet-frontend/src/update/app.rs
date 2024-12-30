use std::{cmp::Ordering, mem};

use yeet_buffer::model::{viewport::ViewPort, Cursor};

use crate::model::{App, Buffer, Window};

pub fn get_focused(app: &App) -> (&ViewPort, &Cursor, &Buffer) {
    let (viewport, cursor, focused_id) = match &app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Content(vp, cursor, it) => (vp, cursor, it),
    };

    match app.buffers.get(&focused_id) {
        Some(it) => return (viewport, cursor, it),
        None => todo!(),
    };
}

pub fn get_focused_mut(app: &mut App) -> (&mut ViewPort, &mut Cursor, &mut Buffer) {
    let (vp, cursor, focused_id) = match &mut app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Content(vp, cursor, it) => (vp, cursor, it),
    };

    match app.buffers.get_mut(&focused_id) {
        Some(it) => return (vp, cursor, it),
        None => todo!(),
    };
}

pub fn get_next_buffer_id(app: &mut App) -> usize {
    let mut next_id = if app.latest_buffer_id >= 9999 {
        1
    } else {
        app.latest_buffer_id + 1
    };

    let mut running_ids: Vec<_> = app.buffers.keys().collect();
    running_ids.sort();

    for id in running_ids {
        match next_id.cmp(&id) {
            Ordering::Equal => next_id += 1,
            Ordering::Greater => break,
            Ordering::Less => {}
        }
    }

    app.latest_buffer_id = next_id;

    next_id
}

pub fn set_focused_buffer(app: &mut App, id: usize) {
    match &mut app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Content(_, _, it) => mem::replace(it, id),
    };
}
