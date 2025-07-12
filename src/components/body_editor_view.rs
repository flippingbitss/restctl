pub fn show(ui: &mut egui::Ui, body: &mut String) {
    let editor = egui::TextEdit::multiline(body)
        .code_editor()
        .min_size(egui::Vec2::splat(200.0))
        .desired_rows(20);
    ui.add(editor);
}
