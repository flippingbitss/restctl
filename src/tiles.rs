use egui_tiles::{Tile, TileId, Tiles};
use log::{debug, info, log};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Pane {
    nr: usize,
}

impl std::fmt::Debug for Pane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("View").field("nr", &self.nr).finish()
    }
}

impl Pane {
    pub fn with_nr(nr: usize) -> Self {
        Self { nr }
    }

    pub fn ui(&self, ui: &mut egui::Ui) -> egui_tiles::UiResponse {
        debug!("rendering pane");
        let color = egui::epaint::Hsva::new(0.103 * self.nr as f32, 0.5, 0.5, 1.0);
        egui::Frame::new().show(ui, |ui| {
            ui.heading(format!("Heading: {}", self.nr));
        });
        // ui.painter().rect_filled(ui.max_rect(), 0.0, color);
        // ui.heading(format!("{}", self.nr));
        // let dragged = ui
        //     .allocate_rect(ui.max_rect(), egui::Sense::click_and_drag())
        //     .on_hover_cursor(egui::CursorIcon::Grab)
        //     .dragged();
        // if dragged {
        //     egui_tiles::UiResponse::DragStarted
        // } else {
        //     egui_tiles::UiResponse::None
        // }

        egui_tiles::UiResponse::None
    }
}

pub struct TreeBehavior {
    pub simplification_options: egui_tiles::SimplificationOptions,
    pub tab_bar_height: f32,
    pub gap_width: f32,
    pub add_child_to: Option<egui_tiles::TileId>,
}

impl Default for TreeBehavior {
    fn default() -> Self {
        Self {
            simplification_options: Default::default(),
            tab_bar_height: 24.0,
            gap_width: 2.0,
            add_child_to: None,
        }
    }
}

impl TreeBehavior {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            simplification_options,
            tab_bar_height,
            gap_width,
            add_child_to: _,
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

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        view: &mut Pane,
    ) -> egui_tiles::UiResponse {
        view.ui(ui)
    }

    fn tab_title_for_pane(&mut self, view: &Pane) -> egui::WidgetText {
        format!("View {}", view.nr).into()
    }

    fn top_bar_right_ui(
        &mut self,
        _tiles: &egui_tiles::Tiles<Pane>,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        _tabs: &egui_tiles::Tabs,
        _scroll_offset: &mut f32,
    ) {
        if ui.button("➕").clicked() {
            self.add_child_to = Some(tile_id);
        }
    }

    // ---
    // Settings:

    fn tab_bar_height(&self, _style: &egui::Style) -> f32 {
        self.tab_bar_height
    }

    fn gap_width(&self, _style: &egui::Style) -> f32 {
        self.gap_width
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
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TabsView {
    pub tree: egui_tiles::Tree<Pane>,

    #[serde(skip)]
    pub behavior: TreeBehavior,
}

impl Default for TabsView {
    fn default() -> Self {
        let mut next_view_nr = 0;
        let mut gen_view = || {
            let view = Pane::with_nr(next_view_nr);
            next_view_nr += 1;
            view
        };

        let mut tiles = egui_tiles::Tiles::default();

        let mut tabs = vec![];
        tabs.push({
            let cells = (0..3).map(|_| tiles.insert_pane(gen_view())).collect();
            tiles.insert_grid_tile(cells)
        });
        tabs.push(tiles.insert_pane(gen_view()));

        let root = tiles.insert_tab_tile(tabs);

        let tree = egui_tiles::Tree::new("my_tree", root, tiles);

        debug!("Tree: {:?}", tree);

        Self {
            tree,
            behavior: Default::default(),
        }
    }
}
