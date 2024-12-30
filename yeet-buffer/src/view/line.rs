use crate::model::{ansi::Ansi, viewport::ViewPort, BufferLine, Cursor, CursorPosition, Mode};

pub fn add_line_styles(
    vp: &ViewPort,
    mode: &Mode,
    cursor: Option<&Cursor>,
    index: &usize,
    line: &mut BufferLine,
) -> Ansi {
    let content_width = vp.get_content_width(line);
    let ansi = line.content.skip_chars(vp.horizontal_index);
    let ansi = add_search_styles(line, &ansi);

    add_cursor_styles(vp, mode, cursor, index, content_width, &ansi)
}

fn add_search_styles(line: &BufferLine, ansi: &Ansi) -> Ansi {
    if let Some(search_char_position) = &line.search_char_position {
        let mut content = ansi.clone();
        for (index, length) in search_char_position.iter() {
            let reset = format!(
                "\x1b[0m{}",
                content.get_ansi_escape_sequences_till_char(*index + 1)
            );

            content.insert(*index, "\x1b[41m");
            content.insert(index + length, &reset);
        }
        content
    } else {
        ansi.clone()
    }
}

fn add_cursor_styles(
    vp: &ViewPort,
    mode: &Mode,
    cursor: Option<&Cursor>,
    index: &usize,
    content_width: usize,
    ansi: &Ansi,
) -> Ansi {
    let mut content = ansi.clone();
    if let Some(cursor) = cursor {
        if cursor.vertical_index - vp.vertical_index != *index {
            return content;
        }

        let char_count = content.count_chars();
        let line_length = if char_count > content_width {
            content_width
        } else if char_count == 0 {
            1
        } else {
            char_count
        };

        let repeat_count = if content_width > line_length {
            content_width - line_length
        } else {
            0
        };
        if cursor.hide_cursor_line {
            content.append(" ".repeat(repeat_count).as_str());
        } else {
            content.prepend("\x1b[100m");
            content.append(" ".repeat(repeat_count).as_str());
            content.append("\x1b[0m");
        };

        if cursor.hide_cursor {
            return content;
        }

        let cursor_index = match &cursor.horizontal_index {
            CursorPosition::End => line_length - 1,
            CursorPosition::None => return content,
            CursorPosition::Absolute {
                current,
                expanded: _,
            } => *current,
        };

        // FIX: reset should just use the ansi code for reset inverse (27)
        // https://github.com/ratatui/ansi-to-tui/issues/50
        let reset = format!(
            "\x1b[0m{}",
            content.get_ansi_escape_sequences_till_char(cursor_index + 1)
        );

        let (code, reset) = match mode {
            Mode::Command(_) | Mode::Normal => ("\x1b[7m", reset.as_str()),
            Mode::Insert => ("\x1b[4m", reset.as_str()),
            Mode::Navigation => ("", ""),
        };

        content.insert(cursor_index, code);
        content.insert(cursor_index + 1, reset);
    }

    content
}
