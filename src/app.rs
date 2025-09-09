const SAMPLE_JSON: &'static str = r#"{"web-app": {
  "servlet": [   
    {
      "servlet-name": "cofaxCDS",
      "servlet-class": "org.cofax.cds.CDSServlet",
      "init-param": {
        "configGlossary:installationAt": "Philadelphia, PA",
        "configGlossary:adminEmail": "ksm@pobox.com",
        "configGlossary:poweredBy": "Cofax",
        "configGlossary:poweredByIcon": "/images/cofax.gif",
        "dataStoreUrl": "jdbc:microsoft:sqlserver://LOCALHOST:1433;DatabaseName=goon",
        "dataStoreUser": "sa",
        "dataStorePassword": "dataStoreTestQuery",
        "dataStoreTestQuery": "SET NOCOUNT ON;select test='test';",
        "dataStoreLogFile": "/usr/local/tomcat/logs/datastore.log",
        "dataStoreInitConns": 10,
        "dataStoreMaxConns": 100,
        "dataStoreConnUsageLimit": 100,
        "dataStoreLogLevel": "debug",
        "maxUrlLength": 500}},
    {
      "servlet-name": "cofaxEmail",
      "servlet-class": "org.cofax.cds.EmailServlet",
      "init-param": {
      "mailHost": "mail1",
      "mailHostOverride": "mail2"}},
    {
      "servlet-name": "cofaxAdmin",
      "servlet-class": "org.cofax.cds.AdminServlet"},
 
    {
      "servlet-name": "fileServlet",
      "servlet-class": "org.cofax.cds.FileServlet"},
    {
      "servlet-name": "cofaxTools",
      "servlet-class": "org.cofax.cms.CofaxToolsServlet",
      "init-param": {
        "templatePath": "toolstemplates/",
        "log": 1,
        "logLocation": "/usr/local/tomcat/logs/CofaxTools.log",
        "logMaxSize": "",
        "dataLog": 1,
        "dataLogLocation": "/usr/local/tomcat/logs/dataLog.log",
        "dataLogMaxSize": "",
        "removePageCache": "/content/admin/remove?cache=pages&id=",
        "removeTemplateCache": "/content/admin/remove?cache=templates&id=",
        "fileTransferFolder": "/usr/local/tomcat/webapps/content/fileTransferFolder",
        "lookInContext": 1,
        "adminGroupID": 4,
        "betaServer": true}}],
  "servlet-mapping": {
    "cofaxCDS": "/",
    "cofaxEmail": "/cofaxutil/aemail/*",
    "cofaxAdmin": "/admin/*",
    "fileServlet": "/static/*",
    "cofaxTools": "/tools/*"},
 
  "taglib": {
    "taglib-uri": "cofax.tld",
    "taglib-location": "/WEB-INF/tlds/cofax.tld"}}}"#;

use egui::{Frame, ThemePreference, util::History};
use ropey::Rope;

use crate::{
    code::{self, builder::JsonSource, editor::BodyEditorView},
    text_edit,
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    frame_history: History<f32>,
    source_text_edit: String,

    #[serde(skip)]
    json_source: code::builder::JsonSource,

    #[serde(skip)]
    body_editor_view: BodyEditorView,
}

impl Default for App {
    fn default() -> Self {
        Self {
            // source: RopeBuffer { rope: Rope::from_str(SAMPLE_JSON) },
            body_editor_view: BodyEditorView::default(),
            json_source: JsonSource::text(String::from(r#"{"user": {"address": {"city": ""}}}"#)),
            source_text_edit: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Mauris vehicula pretium ligula bibendum varius. Nulla diam elit, dictum vitae ultricies quis, pretium non nulla. Integer eget nulla et felis vehicula faucibus vitae eget eros. Nam diam magna, ullamcorper a arcu nec, lobortis vulputate justo. Quisque sed congue lacus. Fusce ullamcorper porttitor aliquam. Donec ultrices scelerisque ligula ut auctor. Maecenas sit amet pharetra urna, at dictum urna. Fusce vel tortor ut purus pellentesque gravida sit amet malesuada urna. Suspendisse id mi eu risus vestibulum feugiat at in ante. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Etiam vitae tincidunt nulla. Aenean eu quam neque. Cras enim sem, viverra sit amet tortor sit amet, aliquet pellentesque nibh.".to_owned(),
            frame_history: History::new(5..20, 2.0),
        }
    }
}

impl App {
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_theme(ThemePreference::Dark);
        let previous_frame_time = frame.info().cpu_usage.unwrap_or_default();
        if let Some(latest) = self.frame_history.latest_mut() {
            *latest = previous_frame_time; // rewrite history now that we know
        }
        let now = ctx.input(|i| i.time);
        self.frame_history.add(now, previous_frame_time);
        self.ui(ctx, frame);
    }
}

impl App {
    fn ui(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .exact_height(32.0)
            .show_separator_line(true)
            .show(ctx, |ui| {
                ui.heading("bottom");
            });

        egui::SidePanel::left("tree").show(ctx, |ui| {
            ui.heading("Debug tools");
            ui.label("CPU Usage:");
            ui.label(format!(
                "{:.2} ms / frame",
                1e3 * self.frame_history.average().unwrap_or_default()
            ));
        });

        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .inner_margin(10)
                    .fill(ctx.style().visuals.panel_fill),
            )
            .show(ctx, |ui| {
                ui.heading("Central Panel");

                let widget = text_edit::TextEdit::multiline(&mut self.source_text_edit)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .desired_rows(5)
                    .frame(false)
                    .hint_text("Enter Code");
                ui.add(widget);

                ui.separator();
                // if (self.source.ends_with("aaaaa")) {
                //     self.source.truncate(self.source.len() - "aaaaa".len());
                // } else {
                //     self.source += "a";
                // }
                self.body_editor_view.show(ui, ctx, &mut self.json_source);
            });
    }
}
