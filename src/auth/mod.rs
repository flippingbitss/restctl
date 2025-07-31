pub(crate) mod sigv4;

use std::str::FromStr;

use base64::Engine;

#[derive(Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AuthLocation {
    #[default]
    Headers,
    Query,
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyParams {
    pub key: String,
    pub value: String,
    pub location: AuthLocation,
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SigV4Params {
    pub access_key: String,
    pub secret_key: String,
    pub session_token: String,
    pub service: String,
    pub region: String,
}

#[derive(Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub enum RequestAuthType {
    #[default]
    None,
    BasicAuth,
    Bearer,
    ApiKey,
    AwsSigV4,
}

impl std::fmt::Display for RequestAuthType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "No auth"),
            Self::BasicAuth => write!(f, "Basic Auth"),
            Self::Bearer => write!(f, "Bearer"),
            Self::ApiKey => write!(f, "API Key"),
            Self::AwsSigV4 => write!(f, "AWS SigV4"),
        }
    }
}

impl std::fmt::Display for RequestAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "No auth"),
            Self::BasicAuth { .. } => write!(f, "Basic Auth"),
            Self::Bearer { .. } => write!(f, "Bearer"),
            Self::ApiKey(..) => write!(f, "API Key"),
            Self::AwsSigV4(..) => write!(f, "AWS SigV4"),
        }
    }
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum RequestAuth {
    #[default]
    None,
    BasicAuth {
        username: String,
        password: String,
    },
    Bearer {
        token: String,
    },
    ApiKey(ApiKeyParams),
    AwsSigV4(SigV4Params),
}

impl RequestAuth {
    pub fn apply(self, request: &mut http::Request<Vec<u8>>) {
        match self {
            RequestAuth::BasicAuth { username, password } => {
                let value = format!("{}:{}", username, password);
                let encoded_value = base64::engine::general_purpose::STANDARD.encode(value);
                request.headers_mut().insert(
                    http::header::AUTHORIZATION,
                    http::HeaderValue::from_str(&format!("Basic {encoded_value}")).unwrap(),
                );
            }
            RequestAuth::Bearer { token } => {
                request.headers_mut().insert(
                    http::header::AUTHORIZATION,
                    http::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
                );
            }
            RequestAuth::ApiKey(params) => match params.location {
                AuthLocation::Headers => {
                    request.headers_mut().insert(
                        http::HeaderName::from_str(&params.key).unwrap(),
                        http::HeaderValue::from_str(&params.value).unwrap(),
                    );
                }
                _ => {} // AuthLocation::Query => request.set_query_param(&params.key, &params.value),
            },
            RequestAuth::AwsSigV4(params) => match sigv4::apply(request, params) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("failed to sign request {}", err);
                }
            },
            RequestAuth::None => {}
        }
    }
}
