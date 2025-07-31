use egui::TextStyle;
use egui::vec2;

use crate::auth::ApiKeyParams;
use crate::auth::AuthLocation;
use crate::auth::RequestAuth;
use crate::auth::SigV4Params;

pub fn show(ui: &mut egui::Ui, auth: &mut RequestAuth) {
    ui.horizontal(|ui| {
        ui.label("Select Auth type: ");
        show_selection_combobox(ui, auth);
    });

    ui.add_space(16.0);

    let item_spacing = ui.style().spacing.item_spacing + egui::Vec2::new(0.0, 6.0);
    egui::Grid::new("auth_params_editor")
        .num_columns(2)
        .min_col_width(60.0)
        .max_col_width(200.0)
        .spacing(item_spacing)
        .show(ui, |ui| match auth {
            RequestAuth::None => {}
            RequestAuth::BasicAuth { username, password } => {
                show_basic_auth(ui, username, password)
            }
            RequestAuth::Bearer { token } => show_bearer(ui, token),
            RequestAuth::ApiKey(params) => show_api_key(ui, params),
            RequestAuth::AwsSigV4(params) => show_sigv4(ui, params),
        });
}

#[rustfmt::skip]
fn show_selection_combobox(ui: &mut egui::Ui, auth: &mut RequestAuth) {
    egui::ComboBox::from_id_salt("auth_selection")
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
}

fn show_sigv4(ui: &mut egui::Ui, params: &mut SigV4Params) {
    for (label, value) in [
        ("Access Key", &mut params.access_key),
        ("Secret Key", &mut params.secret_key),
        ("Session Token", &mut params.session_token),
        ("Region", &mut params.region),
        ("Service", &mut params.service),
    ] {
        ui.label(label);
        ui.add(egui::TextEdit::singleline(value).font(TextStyle::Monospace));
        ui.end_row();
    }
}
fn show_api_key(ui: &mut egui::Ui, params: &mut ApiKeyParams) {
    ui.label("Key");
    ui.add(egui::TextEdit::singleline(&mut params.key).font(TextStyle::Monospace));
    ui.end_row();

    ui.label("Value");
    ui.add(egui::TextEdit::singleline(&mut params.value).font(TextStyle::Monospace));
    ui.end_row();

    ui.end_row();
    ui.label("Location ");
    ui.horizontal(|ui| {
        ui.selectable_value(&mut params.location, AuthLocation::Headers, "Headers");
        ui.selectable_value(&mut params.location, AuthLocation::Query, "Query Params");
    });
    ui.end_row();
}

fn show_bearer(ui: &mut egui::Ui, token: &mut String) {
    ui.label("Token");
    ui.add(egui::TextEdit::multiline(token).code_editor());
    ui.end_row();
}

fn show_basic_auth(ui: &mut egui::Ui, username: &mut String, password: &mut String) {
    ui.label("Username");
    ui.add(egui::TextEdit::singleline(username).font(TextStyle::Monospace));
    ui.end_row();

    ui.label("Password");
    ui.add(egui::TextEdit::singleline(password).font(TextStyle::Monospace));
    ui.end_row();
}
