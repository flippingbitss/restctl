use egui::{Context, Margin};

use crate::{
    core::RequestState,
    http::{self, HttpMethod},
};

pub fn show<'a>(ui: &mut egui::Ui, state: &mut RequestState) {
    // The central panel the region left after adding TopPanel's and SidePanel's
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("http.method")
            .selected_text(format!("{:?}", &state.method))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.method, HttpMethod::Get, "GET");
                ui.selectable_value(&mut state.method, HttpMethod::Head, "HEAD");
                ui.selectable_value(&mut state.method, HttpMethod::Post, "POST");
            });

        ui.add(
            egui::TextEdit::singleline(&mut state.url)
                .margin(Margin::same(6))
                .hint_text("http://httpbin.org/get"),
        );
        if ui.button("Send").clicked() {
            http::execute_with_state(state);
        }
        // ui.selectable_label
        // ui.horizontal(|ui| {
        //     ui.selectable_value(&mut state.method, HttpMethod::Get, "GET");
        //     ui.selectable_value(&mut state.method, HttpMethod::Head, "HEAD");
        //     ui.selectable_value(&mut state.method, HttpMethod::Post, "POST");
        // });
    });
}
