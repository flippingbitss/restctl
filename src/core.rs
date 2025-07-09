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

use std::sync::{Arc, Mutex};

use crate::http::{HttpMethod, HttpResponse};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AppState {
    pub url: String,
    pub method: HttpMethod,
    pub query: Vec<Param>,
    pub headers: Vec<Param>,
    pub body: String,
    pub response: Arc<Mutex<Option<HttpResponse>>>,
}

impl AppState {
    pub fn new(
        url: String,
        method: HttpMethod,
        query: Vec<Param>,
        headers: Vec<Param>,
        body: String,
        response: Arc<Mutex<Option<HttpResponse>>>,
    ) -> Self {
        Self {
            url,
            method,
            query,
            headers,
            body,
            response,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            url: String::new(),
            body: String::new(),
            method: HttpMethod::Get,
            // query: vec![Default::default()],
            query: vec![
                Param {
                    enabled: true,
                    key: "aaa".into(),
                    value: "value 1".into(),
                },
                Param {
                    enabled: true,
                    key: "bbb".into(),
                    value: "value 2".into(),
                },
                Param {
                    enabled: true,
                    key: "ccc".into(),
                    value: "value 3".into(),
                },
            ],
            headers: vec![Default::default()],
            response: Arc::new(Mutex::new(None)),
        }
    }
}

struct ContainerId(usize);
struct ViewId(usize);

enum Item {
    Container(ContainerId),
    View(ViewId),
}
