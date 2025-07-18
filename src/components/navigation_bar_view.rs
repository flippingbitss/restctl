use egui::WidgetText;

use crate::core::RequestId;

pub struct NavBarTabsBehavior {
    pub simplification_options: egui_tiles::SimplificationOptions,
}

impl egui_tiles::Behavior<RequestId> for NavBarTabsBehavior {
    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        self.simplification_options
    }
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        pane: &mut RequestId,
    ) -> egui_tiles::UiResponse {
        ui.label("test");

        egui_tiles::UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &RequestId) -> egui::WidgetText {
        format!("Request: {}", pane.0).into()
    }
}
