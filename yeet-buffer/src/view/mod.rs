use ansi_to_tui::IntoText;
use ratatui::{
    prelude::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::model::{ansi::Ansi, viewport::ViewPort, BufferLine, Cursor, Mode, TextBuffer};

mod line;
mod prefix;
mod style;

pub fn view(
    viewport: &ViewPort,
    cursor: Option<&Cursor>,
    mode: &Mode,
    buffer: &TextBuffer,
    frame: &mut Frame,
    horizontal_offset: u16,
    vertical_offset: u16,
) {
    let rendered = get_rendered_lines(viewport, buffer);
    let styled = get_styled_lines(viewport, mode, cursor, rendered);

    let rect = Rect {
        x: horizontal_offset,
        y: vertical_offset,
        width: viewport.width,
        height: viewport.height,
    };

    let rect = if viewport.show_border {
        let block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::Black));

        let inner = block.inner(rect);

        frame.render_widget(block, rect);

        inner
    } else {
        rect
    };

    frame.render_widget(Paragraph::new(styled), rect);
}

fn get_rendered_lines(viewport: &ViewPort, buffer: &TextBuffer) -> Vec<BufferLine> {
    buffer
        .lines
        .iter()
        .skip(viewport.vertical_index)
        .take(usize::from(viewport.height))
        .map(|line| line.to_owned())
        .collect()
}

fn get_styled_lines<'a>(
    vp: &ViewPort,
    mode: &Mode,
    cursor: Option<&Cursor>,
    lines: Vec<BufferLine>,
) -> Vec<Line<'a>> {
    let lines = if lines.is_empty() {
        vec![BufferLine::default()]
    } else {
        lines
    };

    let mut result = Vec::new();
    for (i, mut bl) in lines.into_iter().enumerate() {
        let corrected_index = i + vp.vertical_index;

        let content = Ansi::new("")
            .join(&prefix::get_signs(vp, &bl))
            .join(&prefix::get_line_number(vp, corrected_index, cursor))
            .join(&prefix::get_custom_prefix(&bl))
            .join(&prefix::get_border(vp))
            .join(&line::add_line_styles(vp, mode, cursor, &i, &mut bl));

        if let Ok(text) = content.to_string().into_text() {
            result.push(text.lines);
        }
    }

    result.into_iter().flatten().collect()
}
