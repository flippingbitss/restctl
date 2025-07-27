mod builder;
mod output;
mod state;
mod text_buffer;

pub use {
    builder::TextEdit, egui::text_selection::TextCursorState, output::TextEditOutput,
    state::TextEditState, text_buffer::TextBuffer,
};
