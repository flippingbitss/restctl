use std::sync::{Arc, Mutex};

use egui::{panel::TopBottomSide, Ui};

use crate::{
    core::Param,
    http::{self, HttpMethod, HttpRequest, HttpResponse},
    widgets,
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
}

impl Default for App {
    fn default() -> Self {
        Self {
            url: String::new(),
            body: String::new(),
            method: HttpMethod::Get,
            query: vec![Default::default()],
            headers: vec![Default::default()],
            response: Arc::new(Mutex::new(None)),
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("query params");
            widgets::key_value_editor("query_params", &mut self.query, ui);
        });
        let bottom_panel = egui::TopBottomPanel::new(TopBottomSide::Bottom, "bottom_bar")
            .resizable(true)
            .height_range(egui::Rangef::new(100.0, 300.0));

        bottom_panel.show(ctx, |ui| {
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
                    ui.label(format!("Status: {} {}", resp.status, resp.status_text));
                    ui.label(format!(
                        "Response body: {:?}",
                        std::str::from_utf8(&resp.body)
                    ));
                } else {
                    ui.label("No response");
                }
            });
            // ui.add(
            //     egui::TextEdit::multiline(&mut self.response)
            //         .desired_width(f32::INFINITY)
            //         .font(egui::TextStyle::Monospace.resolve(ui.style())),
            // );
        });
    }
}

impl App {
    fn execute_http(&mut self, egui_ctx: &egui::Context) {
        let input = HttpRequest {
            url: self.url.clone(),
            method: self.method,
            query: vec![],
            headers: vec![],
            body: None,
        };

        let response_store = self.response.clone();
        http::execute(input, move |result| match result {
            Ok(resp) => {
                *response_store.lock().unwrap() = Some(resp);
            }
            Err(_) => {}
        });
    }
}
