pub fn show(ui: &mut egui::Ui, body: &mut String) {
    let mut theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
    let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
        let mut layout_job = egui_extras::syntax_highlighting::highlight(
            ui.ctx(),
            ui.style(),
            &theme,
            buf.as_str(),
            "json",
        );
        layout_job.wrap.max_width = wrap_width;
        ui.fonts(|f| f.layout_job(layout_job))
    };

    egui::ScrollArea::vertical()
        .min_scrolled_height(ui.available_height())
        .max_height(ui.available_height())
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(body)
                    .code_editor()
                    .desired_rows(20)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter),
            );
        });
}
