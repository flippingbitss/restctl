use std::sync::{Arc, Mutex};

use egui::{panel::TopBottomSide, CornerRadius, Shadow};
use egui_tiles::Tree;

use crate::{
    core::Param,
    http::{self, HttpMethod, HttpRequest, HttpResponse},
    widgets::{
        RequestPane, RequestPaneKind, RequestTreeBehavior, ResponsePane, ResponsePaneKind,
        ResponseTreeBehavior,
    },
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    url: String,
    method: HttpMethod,
    query: Vec<Param>,
    headers: Vec<Param>,
    body: String,
    response: Arc<Mutex<Option<HttpResponse>>>,

    request_tree: Tree<RequestPane>,
    response_tree: Tree<ResponsePane>,
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

        Self {
            url: String::new(),
            body: String::new(),
            method: HttpMethod::Get,
            query: vec![Default::default()],
            headers: vec![Default::default()],
            response: Arc::new(Mutex::new(None)),
            response_tree,
            request_tree,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

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
                    ui.label("URL:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.url)
                            .hint_text("http://httpbin.org/get"),
                    );
                    ui.end_row();
                    ui.label("Method:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.method, HttpMethod::Get, "GET");
                        ui.selectable_value(&mut self.method, HttpMethod::Head, "HEAD");
                        ui.selectable_value(&mut self.method, HttpMethod::Post, "POST");
                    });
                    ui.end_row();

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
                        ui.label(format!("query: {:?}", self.query));
                        ui.label(format!("URL: {}", self.url));
                        ui.label(format!("Method: {:?}", self.method));
                        ui.label(format!("Body: {}", self.body));
                        ui.separator();

                        let store = self.response.clone();
                        let resp = &*store.lock().unwrap();

                        if let Some(resp) = resp {
                            self.response_ui(resp, ui, ctx);
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
                //     egui::TextEdit::multiline(&mut self.response)
                //         .desired_width(f32::INFINITY)
                //         .font(egui::TextStyle::Monospace.resolve(ui.style())),
                // );
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut behavior = RequestTreeBehavior {
                query: &mut self.query,
                headers: &mut self.headers,
            };
            self.request_tree.ui(&mut behavior, ui);
        });
    }
}

impl App {
    fn response_ui(&mut self, response: &HttpResponse, ui: &mut egui::Ui, ctx: &egui::Context) {
        // let response = &*self.response.lock().unwrap();
        // if let Some(response) = response {
        egui::Frame::window(ui.style())
            .shadow(Shadow::NONE)
            .corner_radius(CornerRadius::ZERO)
            .show(ui, |ui| {
                let mut behavior = ResponseTreeBehavior { response };
                self.response_tree.ui(&mut behavior, ui);
            });
        // }
    }

    fn execute_http(&mut self, _: &egui::Context) {
        let input = HttpRequest {
            url: self.url.clone(),
            method: self.method,
            query: self
                .query
                .iter()
                .filter(|p| p.enabled && !p.key.is_empty() && !p.value.is_empty())
                .map(|p| (p.key.clone(), p.value.clone()))
                .collect(),
            headers: self
                .query
                .iter()
                .filter(|p| p.enabled && !p.key.is_empty() && !p.value.is_empty())
                .map(|p| (p.key.clone(), p.value.clone()))
                .collect(),
            body: None,
        };

        log::info!("{:?}", input);
        let response_store = self.response.clone();
        http::execute(input, move |result| match result {
            Ok(resp) => {
                *response_store.lock().unwrap() = Some(resp);
            }
            Err(_) => {}
        });
    }
}
