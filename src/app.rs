use egui::{Frame, TextWrapMode, ThemePreference};

use crate::{
    components::{body_reader_view::BodyReaderView, params_editor_view::ParamsEditorView},
    core::{RequestId, RequestState},
    header,
    tiles::{Pane, PaneKind, TreeBehavior},
};

#[derive(Hash, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum StateId {
    Request,
}
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    state: Vec<(RequestId, RequestState)>,

    active_request_id: RequestId,

    // navigation_tree: egui_tiles::Tree<RequestId>,
    #[serde(skip)]
    request_tree: egui_tiles::Tree<Pane>,

    #[serde(skip)]
    params_view: ParamsEditorView,

    #[serde(skip)]
    body_reader_view: BodyReaderView,
}

impl Default for App {
    fn default() -> Self {
        let mut next_view_nr = 1;
        let mut gen_view = |kind: PaneKind| {
            let view = Pane::from_values(next_view_nr, kind);
            next_view_nr += 1;
            view
        };
        let mut tiles = egui_tiles::Tiles::default();
        let mut request_tabs = vec![];
        request_tabs.push({
            let left = tiles.insert_pane(gen_view(PaneKind::QueryParams));
            let middle = tiles.insert_pane(gen_view(PaneKind::Headers));
            let right = tiles.insert_pane(gen_view(PaneKind::Body));
            tiles.insert_horizontal_tile(vec![left, middle, right])
        });

        let mut response_tabs = vec![];
        response_tabs.push({
            let left = tiles.insert_pane(gen_view(PaneKind::ResponseStats));
            let middle = tiles.insert_pane(gen_view(PaneKind::ResponseHeaders));
            let right = tiles.insert_pane(gen_view(PaneKind::ResponseBody));

            tiles.insert_horizontal_tile(vec![left, middle, right])
        });

        let request_container = tiles.insert_tab_tile(request_tabs);
        let response_container = tiles.insert_tab_tile(response_tabs);
        let root = tiles.insert_vertical_tile(vec![request_container, response_container]);

        let request_tree = egui_tiles::Tree::new("request_tree", root, tiles);

        let mut state = Vec::with_capacity(10);
        let request_id = RequestId::next();
        state.push((request_id, RequestState::default()));
        Self {
            state: state,
            active_request_id: request_id,
            request_tree: request_tree,
            params_view: Default::default(),
            body_reader_view: Default::default(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // let mut style = cc.egui_ctx.style();
        // cc.egui_ctx.style_mut(|style| {
        // style.spacing.button_padding = Vec2::new(10.0, 6.0);
        // });
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }

    fn empty_ui(&mut self, ui: &mut egui::Ui, _: &egui::Context) {
        ui.horizontal(|ui| {
            ui.label("No requests yet. ");
            if ui.button("Create One").clicked() {
                self.state
                    .push((RequestId::next(), RequestState::default()));
            }
        });
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_theme(ThemePreference::Dark);
        self.ui(ctx);
    }
}

impl App {
    fn ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .exact_height(32.0)
            .show_separator_line(true)
            .show(ctx, |ui| {
                ui.heading("bottom");
            });

        egui::SidePanel::left("tree").show(ctx, |ui| {
            ui.heading("Debug tools");

            ui.collapsing("Tree", |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                let tree_debug = format!("{:#?}", self.request_tree);
                ui.monospace(&tree_debug);
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("Open Requests ({})", self.state.len())).size(16.0),
                );

                if ui.button("New Request").clicked() {
                    self.state.push((RequestId::next(), Default::default()));
                }
            });
            // ui.add(egui::TextEdit::singleline(text).hint_text("Search Requests via URL"));
            ui.separator();

            let row_height = ui.text_style_height(&egui::TextStyle::Body);
            ui.scope(|ui| {
                ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
                egui::ScrollArea::vertical().show_rows(
                    ui,
                    row_height,
                    self.state.len(),
                    |ui, range| {
                        for (request_id, state) in self.state.iter() {
                            let url = if state.url.is_empty() {
                                "<empty>".to_owned()
                            } else {
                                state.url.clone()
                            };
                            let label = format!("{}: {} {}", request_id.0, state.method, url);

                            let width = ui.available_width();
                            if ui
                                .selectable_label(self.active_request_id == *request_id, label)
                                .clicked()
                            {
                                self.active_request_id = *request_id;
                            }
                        }
                    },
                )
            });

            ui.allocate_space(ui.available_size());
            // let area = egui::containers::scroll_area::ScrollArea::vertical();
            // area.show(ui, |ui| {
            //     ui.horizontal_wrapped(|ui| {
            //         let code_points = include_str!(
            //             "../assets/fonts/MaterialSymbolsSharp[FILL,GRAD,opsz,wght].codepoints"
            //         );
            //         for line in code_points.lines() {
            //             let (label, code) = line.split_once(" ").unwrap();
            //             let value = u32::from_str_radix(code, 16).unwrap();
            //             let ch = char::from_u32(value).unwrap();
            //             ui.label(format!("{}", ch));
            //         }
            //     });
            // });
        });

        egui::SidePanel::right("side_panel_right").show(ctx, |ui| {
            ui.heading("Right Panel");
            ui.allocate_space(ui.available_size());
        });

        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .inner_margin(0)
                    .fill(ctx.style().visuals.panel_fill),
            )
            .show(ctx, |ui| {
                if self.state.is_empty() {
                    self.empty_ui(ui, ctx);
                } else {
                    if !self
                        .state
                        .iter()
                        .any(|(id, _)| *id == self.active_request_id)
                    {
                        self.active_request_id = self.state.first().unwrap().0;
                    }
                    self.request_ui(self.active_request_id, ui, ctx);
                }
            });
    }

    fn request_ui(&mut self, request_id: RequestId, ui: &mut egui::Ui, ctx: &egui::Context) {
        let mut state = self.state.iter_mut().find(|el| el.0 == request_id);

        if let Some((_, state)) = state {
            // let response = Self::get_response(state);
            header::show(ui, state);
            let mut tiles_behavior = TreeBehavior::default_with_state(
                state,
                &mut self.params_view,
                &mut self.body_reader_view,
            );
            self.request_tree.ui(&mut tiles_behavior, ui);

            if let Some((tile_id, pane_kind)) = tiles_behavior.add_child_to {
                let pane_id = self
                    .request_tree
                    .tiles
                    .insert_pane(Pane::from_values(101, pane_kind));

                let parent = self.request_tree.tiles.get_mut(tile_id).unwrap();

                match parent {
                    egui_tiles::Tile::Container(container) => {
                        container.add_child(pane_id);
                    }
                    _ => {}
                }
            }
        };
    }
}
