use egui::Ui;

use crate::core::Param;

pub fn key_value_editor(id: &str, values: &mut Vec<Param>, ui: &mut Ui) {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing(egui::Vec2::splat(6.0))
        .striped(true)
        .min_col_width(0.0)
        .max_col_width(200.0)
        .show(ui, |ui| {
            ui.add(egui::Label::new(""));
            ui.add(egui::Label::new("Key"));
            ui.add(egui::Label::new("Value"));
            ui.end_row();
            for i in 0..(values.len()) {
                if let Some(param) = values.get_mut(i) {
                    ui.add(egui::Checkbox::without_text(&mut param.enabled));
                    ui.add(egui::TextEdit::singleline(&mut param.key).desired_width(70.0));
                    ui.add(egui::TextEdit::singleline(&mut param.value).desired_width(200.0));
                    if values.len() > 1 {
                        let close = ui.add(egui::Button::new("X"));
                        if close.clicked() {
                            values.remove(i);
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("X"));
                    }
                    ui.end_row();
                }
            }
        });
    ui.add_space(10.0);
    if ui.button("Add").clicked() {
        values.push(Default::default());
    }
}
