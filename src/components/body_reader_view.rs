use egui_extras::syntax_highlighting::CodeTheme;

#[derive(PartialEq, Eq, Default)]
pub enum BodyReaderViewKind {
    Raw,

    #[default]
    Pretty,
}

#[derive(Default)]
pub struct BodyReaderView {
    kind: BodyReaderViewKind,
}

impl BodyReaderView {
    pub fn show(&mut self, ui: &mut egui::Ui, body: &str, body_pretty: &Option<String>) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.kind, BodyReaderViewKind::Raw, "Raw");
            ui.selectable_value(&mut self.kind, BodyReaderViewKind::Pretty, "Pretty");
        });
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(8.0);

        let body_to_view = match self.kind {
            BodyReaderViewKind::Raw => body,
            BodyReaderViewKind::Pretty => match body_pretty {
                Some(prettified) => prettified,
                None => body,
            },
        };

        egui_extras::syntax_highlighting::code_view_ui(
            ui,
            &CodeTheme::from_style(ui.style()),
            body_to_view,
            "json",
        );
    }
}
