#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Param {
    pub enabled: bool,
    pub key: String,
    pub value: String,
}

impl Default for Param {
    fn default() -> Self {
        Self {
            enabled: true,
            key: Default::default(),
            value: Default::default(),
        }
    }
}

pub trait View {
    fn view(&mut self, ui: &mut egui::Ui);
}
