use egui::{FontId, FontSelection, Layout, Margin, Vec2};

use crate::{
    app::GlobalContext,
    core::RequestState,
    http::{self, HttpMethod},
    tasks,
};

pub fn show<'a>(ui: &mut egui::Ui, state: &mut RequestState, global_context: &GlobalContext) {
    // The central panel the region left after adding TopPanel's and SidePanel's
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.scope(|ui| {
            ui.style_mut().spacing.button_padding = Vec2::new(8.0, 6.5);

            egui::ComboBox::from_id_salt("http.method")
                .selected_text(state.method.to_string())
                .show_ui(ui, |ui| {
                    for method in HttpMethod::values_iter() {
                        ui.selectable_value(&mut state.method, method, method.to_string());
                    }
                });

            ui.allocate_ui_with_layout(
                ui.available_size_before_wrap(),
                Layout::right_to_left(egui::Align::Center),
                |ui| {
                    ui.add_space(10.0);

                    if ui.button("\u{e173}").clicked() {
                        // TODO: impl duplicate request,
                        // one of:
                        //      pass in entire request collection
                        //      send a message upstream
                        //      return a typed response
                    }

                    if ui.button("Send").clicked() {
                        tasks::execute(state, &global_context.async_runtime)
                        // if ui.button("Send").clicked() {
                    }

                    ui.add_sized(
                        ui.available_size_before_wrap(),
                        egui::TextEdit::singleline(&mut state.url)
                            .code_editor()
                            .font(FontSelection::FontId(FontId::monospace(14.0)))
                            .margin(Margin::same(6))
                            .hint_text("http://httpbin.org/get"),
                    );
                },
            );

            // ui.selectable_label
            // ui.horizontal(|ui| {
            //     ui.selectable_value(&mut state.method, HttpMethod::Get, "GET");
            //     ui.selectable_value(&mut state.method, HttpMethod::Head, "HEAD");
            //     ui.selectable_value(&mut state.method, HttpMethod::Post, "POST");
            // });
        });
    });
    ui.add_space(10.0);
}
