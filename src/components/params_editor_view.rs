use crate::core::Param;

#[derive(Default, serde::Deserialize)]
pub struct ParamsEditorView {
    bulk_edit_enabled: bool,
    bulk_edit_value: String,
}

impl ParamsEditorView {
    pub fn show(&mut self, ui: &mut egui::Ui, values: &mut Vec<Param>) {
        ui.horizontal(|ui| {
            if ui
                .checkbox(&mut self.bulk_edit_enabled, "Bulk Edit")
                .clicked()
            {
                if self.bulk_edit_enabled {
                    self.update_bulk_from_key_value(values);
                }
            }

            if ui.button("Delete All").clicked() {
                values.clear();
            }
        });

        ui.add_space(10.0);
        if self.bulk_edit_enabled {
            self.show_bulk_editor(ui, values);
        } else {
            self.show_key_value_editor(ui, values);
        }
        ui.separator();
        self.show_controls(ui, values);
    }

    fn show_bulk_editor(&mut self, ui: &mut egui::Ui, values: &mut Vec<Param>) {
        ui.add(
            egui::TextEdit::multiline(&mut self.bulk_edit_value)
                .font(egui::TextStyle::Monospace)
                .margin(egui::Margin::same(8))
                .hint_text("limit:100\nnext_token:abc")
                .hint_text_font(egui::TextStyle::Monospace),
        );

        if ui.small_button("Save").clicked() {
            self.update_key_value_from_bulk(values);
        }
    }

    fn show_key_value_editor(&mut self, ui: &mut egui::Ui, values: &mut Vec<Param>) {
        let half_spacing_amt = ui.style().spacing.item_spacing.y / 2.0;
        let mut drop_target_result: Option<(usize, usize)> = None;

        for current_item_index in 0..(values.len()) {
            // Render list item and get drag handle's response and list item's response
            let list_item_response = self.list_item_ui(ui, values, current_item_index);
            let drag_response = list_item_response.inner;
            let item_response = list_item_response.response;

            if drag_response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
            }
            if drag_response.drag_started() {
                drag_response.dnd_set_drag_payload(current_item_index);
            }
            // If a drag payload exists, find the target position
            let source_item_index = egui::DragAndDrop::payload::<usize>(ui.ctx()).map(|i| *i);

            self.find_drop_target(
                ui,
                source_item_index,
                current_item_index,
                item_response.rect,
                half_spacing_amt, // we use this offset to get consistent indicator position
                &mut drop_target_result,
            );
        }

        // perform the insertion
        if let Some((source_index, target_index)) = drop_target_result {
            if source_index != target_index {
                let item = values.remove(source_index);
                if source_index < target_index {
                    values.insert(target_index - 1, item)
                } else {
                    values.insert(target_index, item);
                }
            }
        }
    }

    fn show_controls(&mut self, ui: &mut egui::Ui, values: &mut Vec<Param>) {
        // Add new param item
        ui.add_space(10.0);

        ui.vertical(|ui| {
            if ui.button("Add").clicked() {
                values.push(Default::default());
            }
            ui.add_space(10.0);
        });
        //     ui.add_space(ui.available_width());
        // });
    }

    fn list_item_ui(
        &mut self,
        ui: &mut egui::Ui,
        values: &mut Vec<Param>,
        index: usize,
    ) -> egui::InnerResponse<egui::Response> {
        ui.horizontal(|ui| {
            let param = &mut values[index];

            let drag_icon = egui::Label::new("\u{e945}")
                .sense(egui::Sense::empty())
                .selectable(false);

            let drag_handle = ui.add(drag_icon);

            ui.add(egui::Checkbox::without_text(&mut param.enabled));
            ui.add_enabled(
                param.enabled,
                egui::TextEdit::singleline(&mut param.key)
                    .hint_text("Key")
                    .desired_width(150.0),
            );
            ui.add_enabled(
                param.enabled,
                egui::TextEdit::singleline(&mut param.value)
                    .hint_text("Value")
                    .desired_width(200.0),
            );
            if values.len() > 1 {
                let close = ui.add(egui::Button::new("X"));
                if close.clicked() {
                    values.remove(index);
                }
            } else {
                ui.add_enabled(false, egui::Button::new("X"));
            }

            // Check for drag interaction and set drag payload
            // let mut drag_response =
            let drag_response = ui.interact(
                drag_handle.rect,
                drag_handle.id,
                egui::Sense::click_and_drag(),
            );
            drag_response
        })
    }

    /// Find target index to drop item to, sets the result to the passed in 'drop_target_result' param
    fn find_drop_target(
        &mut self,
        ui: &mut egui::Ui,
        source_item_index: Option<usize>,
        current_index: usize,
        current_item_rect: egui::Rect,
        half_item_spacing_amt: f32,
        drop_target_result: &mut Option<(usize, usize)>,
    ) {
        if let Some(source_item_index) = source_item_index {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);

            let (top, bottom) = current_item_rect.split_top_bottom_at_fraction(0.5);
            // pointer in upper half
            let (insert_y, target_index) = if ui.rect_contains_pointer(top) {
                (Some(top.top() - half_item_spacing_amt), Some(current_index))
            // pointer in bottom half
            } else if ui.rect_contains_pointer(bottom) {
                (
                    Some(bottom.bottom() + half_item_spacing_amt),
                    Some(current_index + 1),
                )
            } else {
                (None, None)
            };

            // Found an insert position, draw an indicator there
            if let (Some(insert_y), Some(target_index)) = (insert_y, target_index) {
                ui.painter().hline(
                    ui.cursor()
                        .x_range()
                        .intersection(current_item_rect.x_range()),
                    insert_y,
                    (2.0, egui::Color32::WHITE),
                );

                // note: can't use `response.drag_released()` because we might not be the item which
                // started the drag
                if ui.input(|i| i.pointer.any_released()) {
                    *drop_target_result = Some((source_item_index, target_index));
                    egui::DragAndDrop::clear_payload(ui.ctx());
                }
            }
        };
    }

    fn update_bulk_from_key_value(&mut self, values: &mut Vec<Param>) {
        let bulk_value = values
            .into_iter()
            .map(|param| format!("{}:{}\n", param.key, param.value))
            .collect::<String>();

        self.bulk_edit_value = bulk_value;
    }

    fn update_key_value_from_bulk(&mut self, values: &mut Vec<Param>) {
        if !self.bulk_edit_enabled {
            return;
        }

        let updated = self
            .bulk_edit_value
            .lines()
            .map(|line| line.split_once(':').unwrap_or_else(|| (line, "")))
            .map(|(k, v)| Param::enabled(k.to_owned(), v.to_owned()));

        values.clear();
        values.extend(updated);

        self.bulk_edit_enabled = false;
    }
}
