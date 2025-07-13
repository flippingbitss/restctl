use core::fmt;

use crate::core::{Param, RequestState};

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Head,
    Options,
    Delete,
    Patch,
    Connect,
}

impl HttpMethod {
    pub fn values_iter() -> impl Iterator<Item = HttpMethod> {
        [
            HttpMethod::Get,
            HttpMethod::Post,
            HttpMethod::Put,
            HttpMethod::Patch,
            HttpMethod::Delete,
            HttpMethod::Head,
            HttpMethod::Options,
            HttpMethod::Connect,
        ]
        .into_iter()
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Options => write!(f, "OPTIONS"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Connect => write!(f, "CONNECT"),
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

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
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

#[derive(Debug)]
pub enum HttpError {
    Unknown(String),
}

pub fn execute(
    input: HttpRequest,
    callback: impl 'static + Send + FnOnce(Result<HttpResponse, HttpError>),
) {
    let request = {
        let method = input.method.to_string();
        let url = input.url;
        let body = input.body.unwrap_or_else(|| Vec::new());
        let headers = ehttp::Headers {
            headers: input.headers,
        };
        match input.method {
            HttpMethod::Get => ehttp::Request::get(url),
            HttpMethod::Post => ehttp::Request::post(url, body),
            HttpMethod::Head => ehttp::Request::head(url),
            _ => ehttp::Request {
                method,
                url,
                body,
                headers,
                // mode is required on web
                #[cfg(target_arch = "wasm32")]
                mode: ehttp::Mode::Cors,
            },
        }
    };

    ehttp::fetch(request, |response| {
        let mapped = match response {
            Ok(value) => Ok(HttpResponse::from(value)),
            Err(err) => Err(HttpError::Unknown(err)),
        };
        log::info!("{:?}", mapped);
        callback(mapped);
    });
}

pub fn execute_with_state(state: &mut RequestState) {
    let filter_params = |params: &[Param]| {
        params
            .iter()
            .filter(|p| p.enabled && !p.key.is_empty() && !p.value.is_empty())
            .map(|p| (p.key.clone(), p.value.clone()))
            .collect()
    };

    let input = HttpRequest {
        url: state.url.clone(),
        method: state.method,
        query: filter_params(&state.query),
        headers: filter_params(&state.headers),
        body: Some(state.body.as_bytes().into()),
    };

    log::info!("{:?}", input);
    let response_store = state.response.clone();
    execute(input, move |result| match result {
        Ok(resp) => {
            *response_store.lock().unwrap() = Some(resp);
        }
        Err(resp) => match resp {
            HttpError::Unknown(err) => {
                *response_store.lock().unwrap() = Some(HttpResponse {
                    headers: Default::default(),
                    body: err.into_bytes(),
                    status: 500,
                    status_text: "Unknown failure".into(),
                })
            }
        },
    });
}
