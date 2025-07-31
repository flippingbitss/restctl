pub fn show(ui: &mut egui::Ui, body: &mut String) {
    let editor = egui::TextEdit::multiline(body)
        .code_editor()
        .desired_width(ui.available_width())
        .desired_rows(20);
    ui.add(editor);
}
