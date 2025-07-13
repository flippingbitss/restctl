use egui_extras::syntax_highlighting::CodeTheme;

pub fn show(ui: &mut egui::Ui, body: &str) {
    egui_extras::syntax_highlighting::code_view_ui(
        ui,
        &CodeTheme::from_style(ui.style()),
        body,
        "json",
    );
}
