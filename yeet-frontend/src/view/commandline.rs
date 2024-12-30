use ratatui::{layout::Rect, widgets::Paragraph, Frame};
use yeet_buffer::{model::Mode, view};

use crate::{error::AppError, model::CommandLine};

pub fn view(
    commandline: &CommandLine,
    mode: &Mode,
    frame: &mut Frame,
    vertical_offset: u16,
) -> Result<(), AppError> {
    view::view(
        &commandline.viewport,
        commandline.cursor.as_ref(),
        mode,
        &commandline.buffer,
        frame,
        0,
        vertical_offset,
    );

    let rect = Rect {
        x: commandline.viewport.width,
        y: vertical_offset,
        width: u16::try_from(commandline.key_sequence.chars().count())?,
        height: commandline.viewport.height,
    };

    frame.render_widget(Paragraph::new(commandline.key_sequence.clone()), rect);

    Ok(())
}
