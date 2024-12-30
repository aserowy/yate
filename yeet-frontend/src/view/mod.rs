use ratatui::layout::Rect;

use crate::{error::AppError, model::Model, terminal::TerminalWrapper, update::app};

mod commandline;
mod filetreebuffer;
mod statusline;
mod window;

pub fn model(terminal: &mut TerminalWrapper, model: &Model) -> Result<(), AppError> {
    terminal.draw(|frame| {
        tracing::debug!("Rendering with area: {}", frame.area());

        let vertical_offset = window::view(model, frame).expect("Failed to render window view");
        let (_, cursor, buffer) = app::get_focused(&model.app);

        statusline::view(
            cursor,
            buffer,
            frame,
            Rect {
                x: 0,
                width: frame.area().width,
                y: vertical_offset,
                height: 1,
            },
        );

        commandline::view(
            &model.app.commandline,
            &model.state.modes.current,
            frame,
            vertical_offset + 1,
        )
        .expect("Failed to render commandline view");
    })
}
