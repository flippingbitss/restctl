use core::fmt;
use std::time::{Duration, Instant};

use http::{HeaderValue, Request};

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
    input: Request<Vec<u8>>,
    callback: impl 'static + Send + FnOnce(Result<HttpResponse, HttpError>),
) {
    let request = {
        let headers = input
            .headers()
            .into_iter()
            .map(|(name, value)| {
                (
                    name.to_string(),
                    value.to_str().unwrap_or_default().to_string(),
                )
            })
            .collect::<Vec<(String, String)>>();
        let headers = ehttp::Headers { headers };
        ehttp::Request {
            method: input.method().to_string(),
            url: input.uri().to_string(),
            body: input.into_body(),
            headers,
            // mode is required on web
            #[cfg(target_arch = "wasm32")]
            mode: ehttp::Mode::Cors,
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
