use std::{
    sync::{Arc, Mutex},
    thread,
};

use egui::{
    ahash::HashMap,
    epaint::text::{FontInsert, InsertFontFamily},
    panel::TopBottomSide,
    CornerRadius, Frame, Shadow, ThemePreference, Vec2,
};
use egui_tiles::Tree;

use crate::{
    components::params_editor_view::ParamsEditorView,
    core::{Param, RequestState},
    header,
    http::{self, HttpMethod, HttpRequest, HttpResponse},
    tiles::{Pane, PaneKind, TreeBehavior},
    widgets::{
        RequestPane, RequestPaneKind, RequestTreeBehavior, ResponsePane, ResponsePaneKind,
        ResponseTreeBehavior,
    },
};

#[derive(Hash, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum StateId {
    Request,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    state: RequestState,

    #[serde(skip)]
    request_tree: Tree<RequestPane>,

    #[serde(skip)]
    response_tree: Tree<ResponsePane>,

    #[serde(skip)]
    tree: egui_tiles::Tree<Pane>,

    #[serde(skip)]
    params_view: ParamsEditorView,
}

impl Default for App {
    fn default() -> Self {
        let request_tree = Tree::new_vertical(
            "request_tree",
            vec![
                RequestPane {
                    title: "Headers".to_owned(),
                    kind: RequestPaneKind::Headers,
                },
                RequestPane {
                    title: "Query Params".to_owned(),
                    kind: RequestPaneKind::Query,
                },
            ],
        );

        let response_tree = Tree::new_tabs(
            "response_tree",
            vec![
                ResponsePane {
                    title: "Headers".to_owned(),
                    kind: ResponsePaneKind::Headers,
                },
                ResponsePane {
                    title: "Raw Body".to_owned(),
                    kind: ResponsePaneKind::RawBody,
                },
            ],
        );

        let mut next_view_nr = 1;
        let mut gen_view = || {
            let view = Pane::from_values(next_view_nr, PaneKind::QueryParams);
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
        let state = RequestState::default();

        Self {
            state,
            tree,
            request_tree,
            response_tree,
            params_view: Default::default(),
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
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_theme(ThemePreference::Dark);
        self.layout_ui(ctx);
    }
}

impl App {
    // fn request_state(&mut self) -> &mut AppState {
    //     self.states
    //         .get_mut(&StateId::Request)
    //         .map(|s| s.as_ref())
    //         .expect("no request state")
    // }
    fn layout_ui(&mut self, ctx: &egui::Context) {
        // egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {});

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .exact_height(32.0)
            .show_separator_line(true)
            .show(ctx, |ui| {
                ui.heading("bottom");
            });

        egui::SidePanel::left("tree").show(ctx, |ui| {
            ui.heading("Debug tools");
            // tiles_behavior.ui(ui);

            ui.separator();

            ui.collapsing("Tree", |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                let tree_debug = format!("{:#?}", self.tree);
                ui.monospace(&tree_debug);
            });

            ui.separator();

            if let Some(root) = self.tree.root() {
                // tree_ui(ui, &mut self.tabs.behavior, &mut self.tabs.tree.tiles, root);
            }

            // if let Some(parent) = tiles_behavior.add_child_to.take() {
            //     let new_child = self
            //         .tree
            //         .tiles
            //         .insert_pane(Pane::from_values(100, PaneKind::QueryParams));
            //     if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Tabs(tabs))) =
            //         self.tree.tiles.get_mut(parent)
            //     {
            //         tabs.add_child(new_child);
            //         tabs.set_active(new_child);
            //     }
            // }
        });

        let output = {
            let response = self.state.response.lock().unwrap();
            let resp = &*response;

            if let Some(resp) = resp {
                serde_json::to_string_pretty(&resp).unwrap()
            } else {
                "No response yet".into()
            }
        };

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.heading("right side panel");

            ui.label(output);

            ui.allocate_space(ui.available_size());
        });

        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .inner_margin(0)
                    .fill(ctx.style().visuals.panel_fill),
            )
            .show(ctx, |ui| {
                header::show(ui, &mut self.state);
                // let mut state = self.state.lock().unwrap();
                // ui.group(|ui| {
                //     ui.horizontal_wrapped(|ui| {
                //         let mut state = self.state.lock().unwrap();
                //         let method = state.method;
                //         ui.selectable_value(
                //             &mut state.method,
                //             HttpMethod::Get,
                //             format!("{}", method),
                //         );
                //         ui.text_edit_singleline(&mut state.url);
                //     });
                // });
                //

                let mut tiles_behavior =
                    TreeBehavior::default_with_state(&mut self.state, &mut self.params_view);
                self.tree.ui(&mut tiles_behavior, ui);
            });
    }
}
