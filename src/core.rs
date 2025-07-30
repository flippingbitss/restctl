pub trait View {
    fn view(&mut self, ui: &mut egui::Ui);
}

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

impl RequestState {
    pub fn execute(&mut self) {
        let filter_params = |params: &[Param]| {
            params
                .iter()
                .filter(|p| p.enabled && !p.key.is_empty() && !p.value.is_empty())
                .map(|p| (p.key.clone(), p.value.clone()))
                .collect::<Vec<(String, String)>>()
        };

        let uri_without_query = &self.url;
        let query = serde_urlencoded::to_string(filter_params(&self.query)).unwrap_or_default();
        let full_url = format!("{}?{}", uri_without_query, query);

        let mut request_builder = http::Request::builder()
            .method(http::Method::from_str(&self.method.to_string()).unwrap_or_default())
            .uri(http::Uri::from_str(&full_url).unwrap_or_default())
            // todo move to conditional auto-generated header, keeping for now
            .header(http::header::ACCEPT, HeaderValue::from_static("*/*"));

        for (header_name, header_value) in filter_params(&self.headers) {
            request_builder = request_builder.header(header_name, header_value);
        }
        let mut request = request_builder
            .body(self.body.clone().into_bytes())
            .unwrap();

        self.auth.apply(&mut request);

        log::info!("{:?}", request);
        let response_store = self.response.clone();
        crate::http::execute(request, move |result| match result {
            Ok(resp) => {
                *response_store.lock().unwrap() = Some(resp);
            }
            Err(resp) => match resp {
                HttpError::Unknown(err) => {
                    *response_store.lock().unwrap() = Some(HttpResponse {
                        body_raw: err,
                        ..Default::default()
                    })
                }
            },
        });
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
