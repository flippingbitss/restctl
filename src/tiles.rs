use core::fmt;
use std::ops::Deref;

use egui::{Color32, Id, Margin, Rect, Sense};
use egui_tiles::{SimplificationOptions, Tile, TileId, Tiles};

use crate::{
    components::{
        body_editor_view,
        body_reader_view::{self, BodyReaderView},
        params_editor_view::ParamsEditorView,
        params_reader_view,
    },
    core::RequestState,
};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Pane {
    nr: usize,
    kind: PaneKind,
}

impl std::fmt::Debug for Pane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.nr, self.kind)
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum PaneKind {
    QueryParams,
    Headers,
    Body,
    Auth,
    Script,

    // TODO: (temp) move to its own parent enum
    ResponseBody,
    ResponseHeaders,
}

impl fmt::Display for PaneKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaneKind::QueryParams => write!(f, "Query Params"),
            PaneKind::Headers => write!(f, "Headers"),
            PaneKind::Body => write!(f, "Body"),
            PaneKind::Auth => write!(f, "Auth"),
            PaneKind::Script => write!(f, "Script"),
            PaneKind::ResponseBody => write!(f, "Response Body"),
            PaneKind::ResponseHeaders => write!(f, "Response Headers"),
        }
    }
}

impl Pane {
    pub fn from_values(nr: usize, kind: PaneKind) -> Self {
        Pane { nr, kind }
    }
    pub fn pane_ui(
        &mut self,
        state: &mut RequestState,
        params_view: &mut ParamsEditorView,
        body_reader_view: &mut BodyReaderView,
        ui: &mut egui::Ui,
    ) -> egui_tiles::UiResponse {
        let color = egui::epaint::Hsva::new(0.103 * self.nr as f32, 0.5, 0.5, 1.0);

        let dragged = ui
            .allocate_rect(ui.min_rect(), egui::Sense::click_and_drag())
            .on_hover_cursor(egui::CursorIcon::Grab)
            .dragged();

        egui::Frame::new()
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                match self.kind {
                    PaneKind::QueryParams => {
                        params_view.show(ui, &mut state.query);
                    }
                    PaneKind::Headers => {
                        params_view.show(ui, &mut state.headers);
                    }
                    PaneKind::Body => {
                        body_editor_view::show(ui, &mut state.body);
                    }
                    PaneKind::ResponseBody => {
                        let guard = state.response.lock().unwrap();
                        let response = guard.deref();
                        if let Some(response) = response {
                            body_reader_view.show(ui, &response.body_raw, &response.body_pretty);
                        } else {
                            ui.label("No response yet");
                        }
                    }
                    PaneKind::ResponseHeaders => {
                        let guard = state.response.lock().unwrap();
                        let response = guard.deref();
                        if let Some(response) = response {
                            params_reader_view::show(
                                Id::new("response_headers"),
                                ui,
                                &response.headers,
                            );
                        } else {
                            ui.label("No response yet");
                        }
                    }
                    _ => {}
                }
                ui.allocate_rect(ui.max_rect(), Sense::empty());
            });

        ui.heading(format!("{}", self.nr));
        if dragged {
            egui_tiles::UiResponse::DragStarted
        } else {
            egui_tiles::UiResponse::None
        }
    }
}

pub struct TreeBehavior<'a> {
    pub simplification_options: egui_tiles::SimplificationOptions,
    pub tab_bar_height: f32,
    pub gap_width: f32,
    pub add_child_to: Option<(egui_tiles::TileId, PaneKind)>,
    pub state: &'a mut RequestState,
    // TODO: move these to request view and move req state under it
    pub params_view: &'a mut ParamsEditorView,
    pub body_reader_view: &'a mut BodyReaderView,
}

impl<'a> TreeBehavior<'a> {
    pub fn default_with_state(
        state: &'a mut RequestState,
        params_view: &'a mut ParamsEditorView,
        body_reader_view: &'a mut BodyReaderView,
    ) -> Self {
        Self {
            simplification_options: SimplificationOptions {
                all_panes_must_have_tabs: true,
                ..Default::default()
            },
            tab_bar_height: 24.0,
            gap_width: 2.0,
            add_child_to: None,
            state,
            params_view,
            body_reader_view,
        }
    }
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            simplification_options,
            tab_bar_height,
            gap_width,
            ..
        } = self;

        egui::Grid::new("behavior_ui")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("All panes must have tabs:");
                ui.checkbox(&mut simplification_options.all_panes_must_have_tabs, "");
                ui.end_row();

                ui.label("Join nested containers:");
                ui.checkbox(
                    &mut simplification_options.join_nested_linear_containers,
                    "",
                );
                ui.end_row();

                ui.label("Tab bar height:");
                ui.add(
                    egui::DragValue::new(tab_bar_height)
                        .range(0.0..=100.0)
                        .speed(1.0),
                );
                ui.end_row();

                ui.label("Gap width:");
                ui.add(egui::DragValue::new(gap_width).range(0.0..=20.0).speed(1.0));
                ui.end_row();
            });
    }
}

impl<'a> egui_tiles::Behavior<Pane> for TreeBehavior<'a> {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        view: &mut Pane,
    ) -> egui_tiles::UiResponse {
        view.pane_ui(
            &mut self.state,
            &mut self.params_view,
            &mut self.body_reader_view,
            ui,
        )
    }

    fn tab_title_for_pane(&mut self, view: &Pane) -> egui::WidgetText {
        view.kind.to_string().into()
    }

    fn top_bar_right_ui(
        &mut self,
        _tiles: &egui_tiles::Tiles<Pane>,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        _tabs: &egui_tiles::Tabs,
        _scroll_offset: &mut f32,
    ) {
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            egui::menu::menu_button(ui, "+", |ui| {
                for kind in [
                    PaneKind::QueryParams,
                    PaneKind::Headers,
                    PaneKind::Body,
                    PaneKind::Auth,
                    PaneKind::Script,
                    PaneKind::ResponseBody,
                    PaneKind::ResponseHeaders,
                ] {
                    if ui.selectable_label(false, kind.to_string()).clicked() {
                        self.add_child_to = Some((tile_id, kind));
                    }
                }
            });
            ui.separator();
        });
    }

    // ---
    // Settings:

    fn tab_bar_height(&self, style: &egui::Style) -> f32 {
        self.tab_bar_height + style.spacing.button_padding.y * 2.0
    }

    fn gap_width(&self, _style: &egui::Style) -> f32 {
        1.0
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        self.simplification_options
    }

    fn is_tab_closable(&self, _tiles: &Tiles<Pane>, _tile_id: TileId) -> bool {
        true
    }

    fn on_tab_close(&mut self, tiles: &mut Tiles<Pane>, tile_id: TileId) -> bool {
        if let Some(tile) = tiles.get(tile_id) {
            match tile {
                Tile::Pane(pane) => {
                    // Single pane removal
                    let tab_title = self.tab_title_for_pane(pane);
                    log::debug!("Closing tab: {}, tile ID: {tile_id:?}", tab_title.text());
                }
                Tile::Container(container) => {
                    // Container removal
                    log::debug!("Closing container: {:?}", container.kind());
                    let children_ids = container.children();
                    for child_id in children_ids {
                        if let Some(Tile::Pane(pane)) = tiles.get(*child_id) {
                            let tab_title = self.tab_title_for_pane(pane);
                            log::debug!("Closing tab: {}, tile ID: {tile_id:?}", tab_title.text());
                        }
                    }
                }
            }
        }

        // Proceed to removing the tab
        true
    }

    fn close_button_outer_size(&self) -> f32 {
        12.0
    }

    fn close_button_inner_margin(&self) -> f32 {
        2.0
    }

    fn tab_title_for_tile(&mut self, tiles: &Tiles<Pane>, tile_id: TileId) -> egui::WidgetText {
        if let Some(tile) = tiles.get(tile_id) {
            match tile {
                Tile::Pane(pane) => self.tab_title_for_pane(pane),
                Tile::Container(container) => std::format!("{:?}", container.kind()).into(),
            }
        } else {
            "MISSING TILE".into()
        }
    }

    fn tab_ui(
        &mut self,
        tiles: &mut Tiles<Pane>,
        ui: &mut egui::Ui,
        id: egui::Id,
        tile_id: TileId,
        state: &egui_tiles::TabState,
    ) -> egui::Response {
        let text = self.tab_title_for_tile(tiles, tile_id);
        let close_btn_size = egui::Vec2::splat(self.close_button_outer_size());
        let close_btn_left_padding = 4.0;
        let font_id = egui::TextStyle::Button.resolve(ui.style());
        let galley = text.into_galley(ui, Some(egui::TextWrapMode::Extend), f32::INFINITY, font_id);

        let x_margin = self.tab_title_spacing(ui.visuals());

        let button_width = galley.size().x
            + 2.0 * x_margin
            + f32::from(state.closable) * (close_btn_left_padding + close_btn_size.x);
        let (_, tab_rect) = ui.allocate_space(egui::vec2(button_width, ui.available_height()));

        let tab_response = ui
            .interact(tab_rect, id, Sense::click_and_drag())
            .on_hover_cursor(egui::CursorIcon::Grab);

        // Show a gap when dragged
        if ui.is_rect_visible(tab_rect) && !state.is_being_dragged {
            let bg_color = self.tab_bg_color(ui.visuals(), tiles, tile_id, state);
            let stroke = self.tab_outline_stroke(ui.visuals(), tiles, tile_id, state);
            ui.painter().rect(
                tab_rect.shrink(0.5),
                0.0,
                bg_color,
                stroke,
                egui::StrokeKind::Inside,
            );

            if state.active {
                // Make the tab name area connect with the tab ui area:
                ui.painter().hline(
                    tab_rect.x_range(),
                    tab_rect.bottom(),
                    egui::Stroke::new(stroke.width + 1.0, bg_color),
                );
            }

            // Prepare title's text for rendering
            let text_color = self.tab_text_color(ui.visuals(), tiles, tile_id, state);
            let text_position = egui::Align2::LEFT_CENTER
                .align_size_within_rect(galley.size(), tab_rect.shrink(x_margin))
                .min;

            // Render the title
            ui.painter().galley(text_position, galley, text_color);

            // Conditionally render the close button
            if state.closable {
                let close_btn_rect = egui::Align2::RIGHT_CENTER
                    .align_size_within_rect(close_btn_size, tab_rect.shrink(x_margin));

                // Allocate
                let close_btn_id = ui.auto_id_with("tab_close_btn");
                let close_btn_response = ui
                    .interact(close_btn_rect, close_btn_id, Sense::click_and_drag())
                    .on_hover_cursor(egui::CursorIcon::Default);

                let visuals = ui.style().interact(&close_btn_response);

                // Scale based on the interaction visuals
                let rect = close_btn_rect
                    .shrink(self.close_button_inner_margin())
                    .expand(visuals.expansion);
                let stroke = visuals.fg_stroke;

                // paint the crossed lines
                ui.painter() // paints \
                    .line_segment([rect.left_top(), rect.right_bottom()], stroke);
                ui.painter() // paints /
                    .line_segment([rect.right_top(), rect.left_bottom()], stroke);

                // Give the user a chance to react to the close button being clicked
                // Only close if the user returns true (handled)
                if close_btn_response.clicked() {
                    log::debug!("Tab close requested for tile: {tile_id:?}");

                    // Close the tab if the implementation wants to
                    if self.on_tab_close(tiles, tile_id) {
                        log::debug!("Implementation confirmed close request for tile: {tile_id:?}");

                        tiles.remove(tile_id);
                    } else {
                        log::debug!("Implementation denied close request for tile: {tile_id:?}");
                    }
                }
            }
        }

        self.on_tab_button(tiles, tile_id, tab_response)
    }

    fn drag_ui(&mut self, tiles: &Tiles<Pane>, ui: &mut egui::Ui, tile_id: TileId) {
        let mut frame = egui::Frame::popup(ui.style());
        frame.fill = frame.fill.gamma_multiply(0.5); // Make see-through
        frame.show(ui, |ui| {
            // TODO(emilk): preview contents?
            let text = self.tab_title_for_tile(tiles, tile_id);
            ui.label(text);
        });
    }

    fn on_tab_button(
        &mut self,
        _tiles: &Tiles<Pane>,
        _tile_id: TileId,
        button_response: egui::Response,
    ) -> egui::Response {
        button_response
    }

    fn retain_pane(&mut self, _pane: &Pane) -> bool {
        true
    }

    fn min_size(&self) -> f32 {
        32.0
    }

    fn preview_dragged_panes(&self) -> bool {
        false
    }

    fn dragged_overlay_color(&self, visuals: &egui::Visuals) -> Color32 {
        visuals.panel_fill.gamma_multiply(0.5)
    }

    fn paint_on_top_of_tile(
        &self,
        _painter: &egui::Painter,
        _style: &egui::Style,
        _tile_id: TileId,
        _rect: Rect,
    ) {
    }

    fn resize_stroke(
        &self,
        style: &egui::Style,
        resize_state: egui_tiles::ResizeState,
    ) -> egui::Stroke {
        match resize_state {
            egui_tiles::ResizeState::Idle => {
                egui::Stroke::new(self.gap_width(style), self.tab_bar_color(&style.visuals))
            }
            egui_tiles::ResizeState::Hovering => style.visuals.widgets.hovered.fg_stroke,
            egui_tiles::ResizeState::Dragging => style.visuals.widgets.active.fg_stroke,
        }
    }

    fn tab_title_spacing(&self, _visuals: &egui::Visuals) -> f32 {
        8.0
    }

    fn tab_bar_color(&self, visuals: &egui::Visuals) -> Color32 {
        if visuals.dark_mode {
            visuals.widgets.active.weak_bg_fill
        } else {
            (egui::Rgba::from(visuals.panel_fill) * egui::Rgba::from_gray(0.8)).into()
        }
    }

    fn tab_bg_color(
        &self,
        visuals: &egui::Visuals,
        _tiles: &Tiles<Pane>,
        _tile_id: TileId,
        state: &egui_tiles::TabState,
    ) -> Color32 {
        // if state.active {
        //     visuals.panel_fill // same as the tab contents
        // } else {
        Color32::TRANSPARENT // fade into background
        // }
    }

    fn tab_outline_stroke(
        &self,
        visuals: &egui::Visuals,
        _tiles: &Tiles<Pane>,
        _tile_id: TileId,
        state: &egui_tiles::TabState,
    ) -> egui::Stroke {
        egui::Stroke::NONE
    }

    fn tab_bar_hline_stroke(&self, visuals: &egui::Visuals) -> egui::Stroke {
        egui::Stroke::new(1.0, visuals.widgets.noninteractive.fg_stroke.color)
    }

    fn tab_text_color(
        &self,
        visuals: &egui::Visuals,
        _tiles: &Tiles<Pane>,
        _tile_id: TileId,
        state: &egui_tiles::TabState,
    ) -> Color32 {
        if state.active {
            visuals.widgets.open.text_color()
        } else {
            visuals.widgets.noninteractive.text_color()
        }
    }

    fn drag_preview_stroke(&self, visuals: &egui::Visuals) -> egui::Stroke {
        visuals.selection.stroke
    }

    fn drag_preview_color(&self, visuals: &egui::Visuals) -> Color32 {
        visuals.selection.stroke.color.gamma_multiply(0.5)
    }

    fn paint_drag_preview(
        &self,
        visuals: &egui::Visuals,
        painter: &egui::Painter,
        parent_rect: Option<Rect>,
        preview_rect: Rect,
    ) {
        let preview_stroke = self.drag_preview_stroke(visuals);
        let preview_color = self.drag_preview_color(visuals);

        if let Some(parent_rect) = parent_rect {
            // Show which parent we will be dropped into
            painter.rect_stroke(parent_rect, 1.0, preview_stroke, egui::StrokeKind::Inside);
        }

        painter.rect(
            preview_rect,
            1.0,
            preview_color,
            preview_stroke,
            egui::StrokeKind::Inside,
        );
    }
    fn ideal_tile_aspect_ratio(&self) -> f32 {
        4.0 / 3.0
    }

    fn on_edit(&mut self, _edit_action: egui_tiles::EditAction) {}
}
