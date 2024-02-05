use std::cmp::{max, min};

use crossterm::event::KeyCode;
use ratatui::{buffer::Buffer, layout::Rect, text::Line, widgets::{Block, Widget}};

#[derive(Clone)]
pub struct TextInput<'a> {
    block: Option<Block<'a>>,
    input: String,
    cursor_position: usize,
    title: &'a str,
}

impl<'a> TextInput<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            block: None,
            input: String::new(),
            cursor_position: 0,
            title,
        }
    }

    pub fn text(&self) -> &str {
        &self.input
    }

    pub fn clear(&mut self) {
        self.input = String::new();
        self.cursor_position = 0;
    }

    pub fn block(mut self, block: Block<'a>) -> TextInput<'a> {
        self.block = Some(block);
        self
    }

    pub fn on_input(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::Char(char_to_insert) => {
                self.on_input_char(char_to_insert)
            }
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Backspace => self.backspace_char(),
            KeyCode::Delete => self.delete_char(),
            _ => {}
        }
    }

    pub fn on_input_char(&mut self, char_to_insert: char) {
        self.input.insert(self.cursor_position, char_to_insert);
        self.cursor_position += 1;
    }

    pub fn move_left(&mut self) {
        let cursor_next_position = self.cursor_position.saturating_sub(1);
        self.cursor_position = max(cursor_next_position, 0);
    }

    pub fn move_right(&mut self) {
        let cursor_next_position = self.cursor_position.saturating_add(1);
        self.cursor_position = min(cursor_next_position, self.input.len());
    }

    pub fn backspace_char(&mut self) {
        if self.cursor_position != 0 {
            let current_index = self.cursor_position;
            let from_left_to_cursor_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_cursor_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_left();
        }
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position < self.input.len() {
            let current_index = self.cursor_position;
            let from_right_to_cursor_index = current_index + 1;

            let after_char_to_delete = self.input.chars().skip(from_right_to_cursor_index);
            let before_char_to_delete = self.input.chars().take(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
        }
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }
}

impl<'a> Widget for TextInput<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let text_area = match self.block.take() {
            Some(b) => {
                let block = b.title(self.title);
                let inner_area = block.inner(area);
                block.render(area, buf);
                inner_area
            }
            None => area,
        };

        if text_area.height < 1 {
            return;
        }

        // buf.set_string(area.x, area.y, &self.input, Style::default());
        buf.set_line(area.x + 1, area.y + 1, &Line::from(self.input), area.width);
    }
}