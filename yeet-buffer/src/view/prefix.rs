use std::cmp::Reverse;

use crate::model::{
    ansi::Ansi,
    viewport::{LineNumber, ViewPort},
    BufferLine, Cursor,
};

pub fn get_border(vp: &ViewPort) -> Ansi {
    Ansi::new(&" ".repeat(vp.get_border_width()))
}

pub fn get_custom_prefix(line: &BufferLine) -> Ansi {
    if let Some(prefix) = &line.prefix {
        Ansi::new(prefix)
    } else {
        Ansi::new("")
    }
}

pub fn get_line_number(vp: &ViewPort, index: usize, cursor: Option<&Cursor>) -> Ansi {
    if vp.line_number == LineNumber::None {
        return Ansi::new("");
    }

    let width = vp.get_line_number_width();
    let number = {
        let number_string = (index + 1).to_string();
        if let Some(index) = number_string.char_indices().nth_back(width - 1) {
            number_string[index.0..].to_string()
        } else {
            number_string
        }
    };

    if let Some(cursor) = cursor {
        if cursor.vertical_index == index {
            return Ansi::new(&format!("\x1b[1m{:<width$}\x1b[0m", number));
        }
    }

    match vp.line_number {
        LineNumber::Absolute => Ansi::new(&format!("{:>width$} ", number)),
        LineNumber::None => Ansi::new(""),
        LineNumber::Relative => {
            if let Some(cursor) = cursor {
                let relative = if cursor.vertical_index > index {
                    cursor.vertical_index - index
                } else {
                    index - cursor.vertical_index
                };

                Ansi::new(&format!("\x1b[90m{:>width$}\x1b[0m", relative))
            } else {
                Ansi::new(&format!("{:>width$}", number))
            }
        }
    }
}

pub fn get_signs(vp: &ViewPort, bl: &BufferLine) -> Ansi {
    let max_sign_count = vp.sign_column_width;

    let mut filtered: Vec<_> = bl
        .signs
        .iter()
        .filter(|s| !vp.hidden_sign_ids.contains(&s.id))
        .collect();

    filtered.sort_unstable_by_key(|s| Reverse(s.priority));

    let signs_string = filtered
        .iter()
        .take(max_sign_count)
        .fold("".to_string(), |acc, s| {
            format!("{}{}{}\x1b[0m", acc, s.style, s.content)
        });

    let signs = Ansi::new(&signs_string);
    let char_count = signs.count_chars();
    if char_count < max_sign_count {
        Ansi::new(&format!(
            "{}{}",
            signs,
            " ".repeat(max_sign_count - char_count)
        ))
    } else {
        signs
    }
}
