use egui::TextStyle;

use crate::auth::ApiKeyParams;
use crate::auth::RequestAuth;
use crate::auth::SigV4Params;

pub fn show(ui: &mut egui::Ui, auth: &mut RequestAuth) {
    #[rustfmt::skip]
    egui::ComboBox::from_label("Select Auth")
        .selected_text(auth.to_string())
        .show_ui(ui, |ui| {
            if ui.selectable_label(matches!(auth, RequestAuth::None { .. }), "No Auth").clicked() {
                *auth = RequestAuth::None;
            }
            if ui.selectable_label(matches!(auth, RequestAuth::BasicAuth { .. }), "Basic Auth").clicked() {
                *auth = RequestAuth::BasicAuth { username: Default::default(), password: Default::default() };
            }
            if ui.selectable_label(matches!(auth, RequestAuth::Bearer { .. }), "Bearer").clicked() {
                *auth = RequestAuth::Bearer { token: Default::default() };
            }
            if ui.selectable_label(matches!(auth, RequestAuth::ApiKey { .. }), "API Key").clicked() {
                *auth = RequestAuth::ApiKey(Default::default());
            }
            if ui.selectable_label(matches!(auth, RequestAuth::AwsSigV4 { .. }), "AWS SigV4").clicked() {
                *auth = RequestAuth::AwsSigV4(Default::default());
            }
        });

    match auth {
        RequestAuth::None => {}
        RequestAuth::BasicAuth { username, password } => show_basic_auth(ui, username, password),
        RequestAuth::Bearer { token } => show_bearer(ui, token),
        RequestAuth::ApiKey(params) => show_api_key(ui, params),
        RequestAuth::AwsSigV4(params) => show_sigv4(ui, params),
    }
}

fn show_sigv4(ui: &mut egui::Ui, params: &mut SigV4Params) {
    todo!()
}

fn show_api_key(ui: &mut egui::Ui, params: &mut ApiKeyParams) {
    todo!()
}

fn show_bearer(ui: &mut egui::Ui, token: &str) {
    todo!()
}

fn show_basic_auth(ui: &mut egui::Ui, username: &mut String, password: &mut String) {
    ui.horizontal(|ui| {
        ui.label("Username");
        ui.add_space(16.0);
        ui.add(egui::TextEdit::singleline(username).font(TextStyle::Monospace));
    });
    ui.horizontal(|ui| {
        ui.label("Password");
        ui.add_space(16.0);
        ui.add(egui::TextEdit::singleline(password).font(TextStyle::Monospace));
    });
}
