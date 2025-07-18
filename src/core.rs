pub trait View {
    fn view(&mut self, ui: &mut egui::Ui);
}

use std::sync::{Arc, Mutex, atomic::AtomicUsize};

use crate::http::{HttpMethod, HttpResponse, Param};

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
    pub response: Arc<Mutex<Option<HttpResponse>>>,
}

impl RequestState {
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
