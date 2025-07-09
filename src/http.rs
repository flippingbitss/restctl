use core::fmt;

use crate::core::AppState;

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum HttpMethod {
    Get,
    Post,
    Head,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Head => write!(f, "HEAD"),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpMethod,
    pub query: Vec<(String, String)>,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct HttpResponse {
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub status: u16,
    pub status_text: String,
}

impl From<ehttp::Response> for HttpResponse {
    fn from(value: ehttp::Response) -> Self {
        HttpResponse {
            headers: value.headers.headers.clone(),
            body: value.bytes,
            status: value.status,
            status_text: value.status_text,
        }
    }
}

pub enum HttpError {
    Unknown,
}

pub fn execute(
    input: HttpRequest,
    callback: impl 'static + Send + FnOnce(Result<HttpResponse, HttpError>),
) {
    let mut request = match input.method {
        HttpMethod::Get => ehttp::Request::get(input.url),
        HttpMethod::Post => ehttp::Request::post(input.url, input.body.unwrap()),
        HttpMethod::Head => ehttp::Request::head(input.url),
    };
    request.headers = ehttp::Headers {
        headers: input.headers.clone(),
    };
    ehttp::fetch(request, |response| {
        let mapped = match response {
            Ok(value) => Ok(HttpResponse::from(value)),
            Err(_) => Err(HttpError::Unknown),
        };
        callback(mapped);
    });
}

pub fn execute_with_state(state: &mut AppState) {
    let input = HttpRequest {
        url: state.url.clone(),
        method: state.method,
        query: state
            .query
            .iter()
            .filter(|p| p.enabled && !p.key.is_empty() && !p.value.is_empty())
            .map(|p| (p.key.clone(), p.value.clone()))
            .collect(),
        headers: state
            .query
            .iter()
            .filter(|p| p.enabled && !p.key.is_empty() && !p.value.is_empty())
            .map(|p| (p.key.clone(), p.value.clone()))
            .collect(),
        body: None,
    };

    log::info!("{:?}", input);
    let response_store = state.response.clone();
    execute(input, move |result| match result {
        Ok(resp) => {
            *response_store.lock().unwrap() = Some(resp);
        }
        Err(_) => {}
    });
}
