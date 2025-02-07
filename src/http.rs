#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum HttpMethod {
    Get,
    Post,
    Head,
}

#[derive(serde::Deserialize, serde::Serialize)]
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
