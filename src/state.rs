use std::sync::{Arc, Mutex};

use egui_tiles::Tree;

use crate::{
    core::Param,
    http::{HttpMethod, HttpResponse},
    widgets::{RequestPane, ResponsePane},
};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AppState {
    pub url: String,
    pub method: HttpMethod,
    pub query: Vec<Param>,
    pub headers: Vec<Param>,
    pub body: String,
    pub response: Arc<Mutex<Option<HttpResponse>>>,
    pub request_tree: Tree<RequestPane>,
    pub response_tree: Tree<ResponsePane>,
}
