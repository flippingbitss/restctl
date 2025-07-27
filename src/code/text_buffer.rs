use std::ops::Range;

use egui::epaint::text::cursor::CCursor;

use egui::text::CCursorRange;
use ropey::{Rope, RopeBuilder, iter::Chars};

pub struct RopeBuffer {
    pub rope: ropey::Rope,
}

impl RopeBuffer {
    pub fn insert_text(&mut self, text: &str, char_index: usize) {
        self.rope.insert(char_index, text);
    }

    pub fn delete_char_range(&mut self, char_range: Range<usize>) {
        self.rope.remove(char_range);
    }

    pub fn char_range(&mut self, char_range: Range<usize>) -> ropey::RopeSlice<'_> {
        self.rope.slice(char_range)
    }

    pub fn byte_index_from_char_index(&self, char_index: usize) -> usize {
        self.rope.char_to_byte(char_index)
    }

    pub fn char_index_from_byte_index(&self, byte_index: usize) -> usize {
        self.rope.byte_to_char(byte_index)
    }

    pub fn clear(&mut self) {
        self.rope = Rope::new();
    }

    /// Replaces all contents of this string with `text`
    pub fn replace_with(&mut self, text: &str) {
        let mut rope = RopeBuilder::new();
        rope.append(text);
        self.rope = rope.finish();
    }

    pub fn delete_selected(&mut self, cursor_range: &CCursorRange) -> CCursor {
        let [min, max] = cursor_range.sorted_cursors();
        self.delete_selected_ccursor_range([min, max])
    }

    pub fn delete_selected_ccursor_range(&mut self, [min, max]: [CCursor; 2]) -> CCursor {
        self.delete_char_range(min.index..max.index);
        CCursor {
            index: min.index,
            prefer_next_row: true,
        }
    }

    pub fn delete_previous_char(&mut self, ccursor: CCursor) -> CCursor {
        if ccursor.index > 0 {
            let max_ccursor = ccursor;
            let min_ccursor = max_ccursor - 1;
            self.delete_selected_ccursor_range([min_ccursor, max_ccursor])
        } else {
            ccursor
        }
    }

    pub fn delete_next_char(&mut self, ccursor: CCursor) -> CCursor {
        self.delete_selected_ccursor_range([ccursor, ccursor + 1])
    }

    pub fn delete_previous_word(&mut self, max_ccursor: CCursor) -> CCursor {
        let mut min_ccursor = max_ccursor;
        let char_index = max_ccursor.index;
        let chars: Chars<'_> = self.rope.chars_at(char_index).reversed();

        for c in chars {
            if !(c.is_alphanumeric() || c == '_') {
                break;
            }
            min_ccursor.index -= 1;
        }

        self.delete_selected_ccursor_range([min_ccursor, max_ccursor])
    }

    pub fn delete_next_word(&mut self, min_ccursor: CCursor) -> CCursor {
        let mut max_ccursor = min_ccursor;

        let char_index = min_ccursor.index;
        let chars: Chars<'_> = self.rope.chars_at(char_index);

        for c in chars {
            if !(c.is_alphanumeric() || c == '_') {
                break;
            }
            max_ccursor.index += 1;
        }

        self.delete_selected_ccursor_range([min_ccursor, max_ccursor])
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn char_count(&self) -> usize {
        self.rope.len_chars()
    }
}
