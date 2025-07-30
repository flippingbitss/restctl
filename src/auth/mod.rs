pub(crate) mod sigv4;

use std::str::FromStr;

use base64::Engine;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub enum AuthLocation {
    #[default]
    Headers,
    Query,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyParams {
    key: String,
    value: String,
    location: AuthLocation,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct SigV4Params {
    access_key: String,
    secret_key: String,
    session_token: String,
    service: String,
    region: String,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
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

#[derive(Default, serde::Serialize, serde::Deserialize)]
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
    pub fn apply<T>(&self, request: &mut http::Request<T>) {
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
            RequestAuth::AwsSigV4(params) => todo!(),
            RequestAuth::None => {}
        }
    }
}
