use crate::{
    code::{
        autocomplete::CompletionQuery,
        builder::{AutoCompletionItem, AutoCompletionState, JsonSource, TextEdit},
    },
    text_edit::TextBuffer,
};
#[derive(Default)]
pub struct BodyEditorView {
    autocompletion: AutoCompletionState,
}

impl BodyEditorView {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        _: &egui::Context,
        json_source: &mut JsonSource,
        autocomplete_fn: impl Fn(CompletionQuery<'_>) -> Vec<AutoCompletionItem>,
    ) {
        let code_widget = TextEdit::json(json_source, &mut self.autocompletion, autocomplete_fn)
            .desired_width(f32::INFINITY)
            .desired_rows(10);
        code_widget.show(ui);
    }
}
