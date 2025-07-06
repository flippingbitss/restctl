use std::sync::{Arc, Mutex};

use egui::{
    epaint::text::{FontInsert, InsertFontFamily},
    panel::TopBottomSide,
    CornerRadius, Frame, Shadow,
};
use egui_tiles::Tree;

use crate::{
    components::key_value_editor::key_value_editor,
    core::{AppState, Param, SharedState},
    http::{self, HttpMethod, HttpRequest, HttpResponse},
    tiles::{Pane, PaneKind, TabsView, TreeBehavior},
    widgets::{
        RequestPane, RequestPaneKind, RequestTreeBehavior, ResponsePane, ResponsePaneKind,
        ResponseTreeBehavior,
    },
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    state: SharedState,

    #[serde(skip)]
    request_tree: Tree<RequestPane>,

    #[serde(skip)]
    response_tree: Tree<ResponsePane>,

    #[serde(skip)]
    tabs: TabsView,
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
        let state = Arc::new(Mutex::new(AppState::default()));

        let tabs = TabsView {
            tree,
            behavior: TreeBehavior::default_with_state(state.clone()),
        };

        Self {
            state,
            tabs,
            request_tree,
            response_tree,
        }
    }
}

impl App {
    fn add_font(ctx: &egui::Context) {
        let bytes = include_bytes!("../assets/fonts/Inter-VariableFont_opsz,wght.ttf");
        ctx.add_font(FontInsert::new(
            "my_font",
            egui::FontData::from_static(bytes),
            vec![InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: egui::epaint::text::FontPriority::Highest,
            }],
        ));
    }

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::add_font(&cc.egui_ctx);
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
        self.layout_ui(ctx);
    }
}

impl App {
    fn layout_ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .exact_height(32.0)
            .show_separator_line(true)
            .show(ctx, |ui| {
                ui.heading("bottom");
            });

        egui::SidePanel::left("tree").show(ctx, |ui| {
            ui.heading("Debug tools");
            self.tabs.behavior.ui(ui);

            ui.separator();

            ui.collapsing("Tree", |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                let tree_debug = format!("{:#?}", self.tabs.tree);
                ui.monospace(&tree_debug);
            });

            ui.separator();

            if let Some(root) = self.tabs.tree.root() {
                // tree_ui(ui, &mut self.tabs.behavior, &mut self.tabs.tree.tiles, root);
            }

            if let Some(parent) = self.tabs.behavior.add_child_to.take() {
                let new_child = self
                    .tabs
                    .tree
                    .tiles
                    .insert_pane(Pane::from_values(100, PaneKind::QueryParams));
                if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Tabs(tabs))) =
                    self.tabs.tree.tiles.get_mut(parent)
                {
                    tabs.add_child(new_child);
                    tabs.set_active(new_child);
                }
            }
        });

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.heading("right side panel");

            ui.allocate_space(ui.available_size());
        });

        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .inner_margin(0)
                    .fill(ctx.style().visuals.panel_fill),
            )
            .show(ctx, |ui| {
                ui.group(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        let mut state = self.state.lock().unwrap();
                        let method = state.method;
                        ui.selectable_value(
                            &mut state.method,
                            HttpMethod::Get,
                            format!("{}", method),
                        );
                        ui.text_edit_singleline(&mut state.url);
                    });
                });
                self.tabs.tree.ui(&mut self.tabs.behavior, ui);
            });
    }

    fn old_ui(&mut self, ctx: &egui::Context) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        //

        ctx.debug_painter().rect_stroke(
            ctx.screen_rect(),
            CornerRadius::ZERO,
            ctx.style().visuals.window_stroke(),
            egui::StrokeKind::Inside,
        );

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Hello world!");

            egui::Grid::new("request_params")
                .spacing(egui::Vec2::splat(6.0))
                .min_col_width(70.0)
                .num_columns(2)
                .show(ui, |ui| {
                    {
                        let mut state = self.state.lock().unwrap();
                        ui.label("URL:");
                        ui.add(
                            egui::TextEdit::singleline(&mut state.url)
                                .hint_text("http://httpbin.org/get"),
                        );
                        ui.end_row();
                        ui.label("Method:");
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut state.method, HttpMethod::Get, "GET");
                            ui.selectable_value(&mut state.method, HttpMethod::Head, "HEAD");
                            ui.selectable_value(&mut state.method, HttpMethod::Post, "POST");
                        });
                        ui.end_row();
                    }
                    if ui.button("Send").clicked() {
                        self.execute_http(ctx);
                    }
                });
        });

        egui::TopBottomPanel::new(TopBottomSide::Bottom, "bottom_bar")
            .resizable(true)
            // .height_range(egui::Rangef::new(200.0, 300.0))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Response");
                    ui.vertical(|ui| {
                        let state = self.state.lock().unwrap();
                        ui.label(format!("query: {:?}", state.query));
                        ui.label(format!("URL: {}", state.url));
                        ui.label(format!("Method: {:?}", state.method));
                        ui.label(format!("Body: {}", state.body));
                        ui.separator();

                        let store = state.response.clone();
                        let resp = &*store.lock().unwrap();

                        if let Some(resp) = resp {
                            egui::Frame::window(ui.style())
                                .shadow(Shadow::NONE)
                                .corner_radius(CornerRadius::ZERO)
                                .show(ui, |ui| {
                                    let mut behavior = ResponseTreeBehavior { response: resp };
                                    self.response_tree.ui(&mut behavior, ui);
                                });
                            // ui.label(format!("Status: {} {}", resp.status, resp.status_text));
                            // ui.label(format!(
                            //     "Response body: {:?}",
                            //     std::str::from_utf8(&resp.body)
                            // ));
                        } else {
                            ui.label("No response");
                        }
                    });
                });
                // ui.add(
                //     egui::TextEdit::multiline(&mut self.state.response)
                //         .desired_width(f32::INFINITY)
                //         .font(egui::TextStyle::Monospace.resolve(ui.style())),
                // );
            });

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     let mut state = self.state.lock().unwrap();
        //     let mut behavior = RequestTreeBehavior {
        //         query: &mut query,
        //         headers: &mut headers,
        //     };
        //     self.request_tree.ui(&mut behavior, ui);
        // });
    }

    fn response_ui(&mut self, response: &HttpResponse, ui: &mut egui::Ui, ctx: &egui::Context) {
        // let response = &*self.response.lock().unwrap();
        // if let Some(response) = response {
        // }
    }

    fn execute_http(&mut self, _: &egui::Context) {
        let state = self.state.lock().unwrap();
        let input = HttpRequest {
            url: state.url.clone(),
            method: state.method,
            query: state
                .query
                .iter()
                .filter(|p| p.enabled && !p.key.is_empty() && !p.value.is_empty())
                .map(|p| (p.key.clone(), p.value.clone()))
                .collect(),
            headers: state
                .query
                .iter()
                .filter(|p| p.enabled && !p.key.is_empty() && !p.value.is_empty())
                .map(|p| (p.key.clone(), p.value.clone()))
                .collect(),
            body: None,
        };

        log::info!("{:?}", input);
        let response_store = state.response.clone();
        http::execute(input, move |result| match result {
            Ok(resp) => {
                *response_store.lock().unwrap() = Some(resp);
            }
            Err(_) => {}
        });
    }
}
