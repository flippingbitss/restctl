pub fn show(id: egui::Id, ui: &mut egui::Ui, values: &[(String, String)]) {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing(egui::Vec2::splat(6.0))
        .striped(true)
        .min_col_width(100.0)
        .max_col_width(200.0)
        .show(ui, |ui| {
            ui.add(egui::Label::new("Key"));
            ui.add(egui::Label::new("Value"));
            ui.end_row();
            for i in 0..(values.len()) {
                if let Some(param) = values.get(i) {
                    ui.label(param.0.clone());
                    ui.label(param.1.clone());
                    ui.end_row();
                }
            }
        });

    ui.add_space(16.0);
    if ui.button("Copy \u{e173}").clicked() {
        // TODO: impl copy to clipboard
        log::info!("copy headers");
    }
}
