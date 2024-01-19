use ratatui::{prelude::Rect, Frame};

use crate::model::Model;

use super::buffer;

pub fn view(model: &mut Model, frame: &mut Frame, rect: Rect) {
    buffer::view(&model.current_directory, frame, rect);
}
