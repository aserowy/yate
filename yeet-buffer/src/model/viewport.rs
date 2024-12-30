use std::collections::HashSet;

use super::{BufferLine, SignIdentifier};

#[derive(Debug, Default)]
pub struct WindowSettings {
    pub sign_column_width: usize,
}

#[derive(Clone, Debug, Default)]
pub struct ViewPort {
    pub height: u16,
    pub hidden_sign_ids: HashSet<SignIdentifier>,
    pub horizontal_index: usize,
    pub line_number: LineNumber,
    pub line_number_width: usize,
    pub show_border: bool,
    pub sign_column_width: usize,
    pub vertical_index: usize,
    pub width: u16,
}

// TODO: enable with settings
// TODO: refactor into functions
impl ViewPort {
    pub fn get_border_width(&self) -> usize {
        if self.get_prefix_width() > 0 {
            1
        } else {
            0
        }
    }

    pub fn get_content_width(&self, line: &BufferLine) -> usize {
        let offset = self.get_offset_width(line);
        if usize::from(self.width) < offset {
            0
        } else {
            usize::from(self.width) - offset
        }
    }

    pub fn get_line_number_width(&self) -> usize {
        match self.line_number {
            LineNumber::Absolute => self.line_number_width,
            LineNumber::None => 0,
            LineNumber::Relative => self.line_number_width,
        }
    }

    pub fn get_offset_width(&self, line: &BufferLine) -> usize {
        let custom_prefix_width = if let Some(prefix) = &line.prefix {
            prefix.chars().count()
        } else {
            0
        };

        self.get_prefix_width() + self.get_border_width() + custom_prefix_width
    }

    fn get_prefix_width(&self) -> usize {
        self.sign_column_width + self.get_line_number_width()
    }

    pub fn set(&mut self, settings: &WindowSettings) {
        self.sign_column_width = settings.sign_column_width;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum LineNumber {
    Absolute,
    #[default]
    None,
    Relative,
}
