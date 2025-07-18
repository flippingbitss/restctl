use core::fmt;
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::core::RequestState;

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

#[derive(Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct HttpResponse {
    pub headers: Vec<(String, String)>,
    pub body_raw: String,
    pub ok: bool,
    pub status: u16,
    pub status_text: String,

    pub body_pretty: Option<String>,
    pub duration: Duration,
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

    // not supported on wasm32
    // let start = Instant::now();
    ehttp::fetch(request, move |response| {
        // let duration = start.elapsed();

        let mapped = response
            .map(|response| {
                let body = std::str::from_utf8(&response.bytes).unwrap_or_default();
                let parsed = serde_json::from_slice::<serde_json::Value>(&response.bytes);
                let body_pretty = match parsed {
                    Ok(value) => Some(serde_json::to_string_pretty(&value).unwrap()),
                    Err(e) => {
                        log::warn!("failed to parse response body {}", e);
                        None
                    }
                };
                HttpResponse {
                    headers: response.headers.headers,
                    ok: response.ok,
                    status: response.status,
                    status_text: response.status_text,
                    body_raw: body.to_owned(),
                    body_pretty,
                    duration: Default::default(),
                }
            })
            .map_err(|err| HttpError::Unknown(err));

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
                    body_raw: err,
                    ..Default::default()
                })
            }
        },
    });
}
