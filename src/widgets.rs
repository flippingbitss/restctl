use egui::{CornerRadius, Margin, Shadow, Ui};

use crate::{core::Param, http::HttpResponse};

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

#[derive(serde::Deserialize, serde::Serialize)]
pub enum RequestPaneKind {
    Query,
    Headers,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RequestPane {
    pub title: String,
    pub kind: RequestPaneKind,
}
pub struct RequestTreeBehavior<'a> {
    pub query: &'a mut Vec<Param>,
    pub headers: &'a mut Vec<Param>,
}

impl<'a> egui_tiles::Behavior<RequestPane> for RequestTreeBehavior<'a> {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _: egui_tiles::TileId,
        pane: &mut RequestPane,
    ) -> egui_tiles::UiResponse {
        let mut values = match pane.kind {
            RequestPaneKind::Query => &mut self.query,
            RequestPaneKind::Headers => &mut self.headers,
        };
        egui::Frame::window(ui.style())
            .shadow(Shadow::NONE)
            .corner_radius(CornerRadius::ZERO)
            .show(ui, |ui| {
                ui.label(egui::RichText::new(format!("{}", pane.title)).strong());
                ui.separator();
                key_value_editor(&pane.title, &mut values, ui);
            });
        egui_tiles::UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &RequestPane) -> egui::WidgetText {
        pane.title.clone().into()
    }
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ResponsePane {
    pub title: String,
    pub kind: ResponsePaneKind,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum ResponsePaneKind {
    Headers,
    PrettifiedBody,
    RawBody,
}

pub struct ResponseTreeBehavior<'a> {
    pub response: &'a HttpResponse,
}

impl<'a> egui_tiles::Behavior<ResponsePane> for ResponseTreeBehavior<'a> {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        pane: &mut ResponsePane,
    ) -> egui_tiles::UiResponse {
        egui::Frame::default()
            .outer_margin(Margin::same(5))
            .show(ui, |ui| match pane.kind {
                ResponsePaneKind::Headers => {
                    egui::Grid::new("pane_grid")
                        .num_columns(1)
                        .striped(true)
                        .show(ui, |ui| {
                            for header in self.response.headers.iter() {
                                ui.monospace(format!("{}: {}", header.0, header.1));
                                ui.end_row();
                            }
                        });
                }
                ResponsePaneKind::RawBody => {
                    ui.monospace(std::str::from_utf8(&self.response.body).unwrap());
                }
                _ => {
                    ui.monospace(std::str::from_utf8(&self.response.body).unwrap());
                }
            });
        egui_tiles::UiResponse::None
    }
    fn tab_title_for_pane(&mut self, pane: &ResponsePane) -> egui::WidgetText {
        pane.title.clone().into()
    }

    fn tab_outline_stroke(
        &self,
        visuals: &egui::Visuals,
        _tiles: &egui_tiles::Tiles<ResponsePane>,
        _tile_id: egui_tiles::TileId,
        state: &egui_tiles::TabState,
    ) -> egui::Stroke {
        egui::Stroke::NONE
        // if state.active {
        //     Stroke::new(1.0, visuals.widgets.active.bg_fill)
        // } else {
        //     Stroke::NONE
        // }
    }
}
