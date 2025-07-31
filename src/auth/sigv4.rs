use aws_credential_types::Credentials;
use aws_sigv4::{
    http_request::{SignableBody, SignableRequest, SigningSettings, sign},
    sign::v4,
};

use crate::auth::SigV4Params;

pub fn apply(
    request: &mut http::Request<Vec<u8>>,
    params: SigV4Params,
) -> Result<(), Box<dyn std::error::Error>> {
    let region = params.region;
    let service = params.service;

    let creds = Credentials::new(
        params.access_key,
        params.secret_key,
        Some(params.session_token),
        None,
        "test_provider",
    );
    let identity = creds.into();

    // aws sdk doesn't support wasm as it asks for SystemTime::now()
    // here we construct std::time::SystemTime from the web_time's timestamp
    // on wasm
    let timestamp_as_duration = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap();
    let std_time = std::time::SystemTime::UNIX_EPOCH + timestamp_as_duration;

    let signing_settings = SigningSettings::default();
    let signing_params = v4::SigningParams::builder()
        .identity(&identity)
        .region(&region)
        .name(&service)
        .time(std_time)
        .settings(signing_settings)
        .build()?
        .into();

    let signable_request = SignableRequest::new(
        request.method().as_str(),
        request.uri().to_string(),
        request
            .headers()
            .iter()
            .map(|(k, v)| (k.as_str(), std::str::from_utf8(v.as_bytes()).unwrap())),
        SignableBody::Bytes(&request.body()),
    )?;

    // Sign the request
    let (signing_instructions, _signature) = sign(signable_request, &signing_params)?.into_parts();
    signing_instructions.apply_to_request_http1x(request);

    Ok(())
}
