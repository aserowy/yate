use ratatui::Frame;

use crate::{
    error::AppError,
    model::{Buffer, Model, Window},
};

use super::filetreebuffer;

pub fn view(model: &Model, frame: &mut Frame) -> Result<u16, AppError> {
    // NOTE: extract current shown windows with vp/cursor and buffer id
    let single_window = &model.app.window;

    let (vp, cursor, id) = match &single_window {
        Window::Horizontal(_, _) => todo!(),
        Window::Content(vp, cursor, id) => (vp, cursor, id),
    };

    let buffer = model.app.buffers.get(id).expect("asdf");
    match buffer {
        Buffer::FileTree(it) => {
            filetreebuffer::view(&model.state.modes.current, vp, cursor, it, frame, 0, 0)
        }
        Buffer::_Text(_) => todo!(),
    };

    return single_window.get_height();
}
