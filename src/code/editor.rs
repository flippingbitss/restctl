use crate::{
    code::builder::{AutoCompletionUiState, JsonSource, TextEdit},
    text_edit::TextBuffer,
};
#[derive(Default)]
pub struct BodyEditorView {
    autocompletion: AutoCompletionUiState,
}

impl BodyEditorView {
    pub fn show(&mut self, ui: &mut egui::Ui, _: &egui::Context, json_source: &mut JsonSource) {
        let code_widget = TextEdit::json(json_source, &mut self.autocompletion)
            .desired_width(f32::INFINITY)
            .desired_rows(10);
        code_widget.show(ui);
    }
}
