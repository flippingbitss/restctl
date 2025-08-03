pub trait View {
    fn view(&mut self, ui: &mut egui::Ui);
}

#[cfg(not(target_arch = "wasm32"))]
use std::thread;
use std::{
    str::FromStr,
    sync::{Arc, Mutex, atomic::AtomicUsize},
};

use http::{HeaderValue, Method, Uri, request, uri::PathAndQuery};

use crate::{
    auth::{RequestAuth, RequestAuthType},
    http::{HttpError, HttpMethod, HttpResponse},
};

static ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn get_new_id() -> usize {
    ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::AcqRel)
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct RequestId(pub usize);

impl RequestId {
    pub fn next() -> Self {
        Self(get_new_id())
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RequestState {
    pub url: String,
    pub method: HttpMethod,
    pub query: Vec<Param>,
    pub headers: Vec<Param>,
    pub body: String,
    pub auth: RequestAuth,
    pub response: Arc<Mutex<Option<HttpResponse>>>,
}

impl Default for RequestState {
    fn default() -> Self {
        RequestState {
            // url: String::new(),
            url: "http://httpbin.org/get".to_owned(),
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
            auth: Default::default(),
            response: Arc::new(Mutex::new(None)),
        }
    }
}

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

impl Param {
    pub fn enabled(key: String, value: String) -> Self {
        Param {
            enabled: true,
            key,
            value,
        }
    }
}
