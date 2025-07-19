use crate::http::HttpResponse;

pub fn show(id: egui::Id, ui: &mut egui::Ui, response: &HttpResponse) {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing(egui::Vec2::splat(6.0))
        .striped(true)
        .min_col_width(100.0)
        .max_col_width(200.0)
        .show(ui, |ui| {
            ui.label("OK");
            ui.label(response.ok.to_string());
            ui.end_row();

            ui.label("Status Code");
            ui.label(response.status.to_string());
            ui.end_row();

            ui.label("Status Text");
            ui.label(response.status_text.to_string());
            ui.end_row();
        });
}
