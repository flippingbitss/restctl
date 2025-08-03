use std::str::FromStr;

use http::HeaderValue;

use crate::{async_runtime::AsyncRuntimeHandle, core::Param, http::HttpResponse};

pub fn execute(state: &mut crate::core::RequestState, runtime_handle: &AsyncRuntimeHandle) {
    let uri_without_query = &state.url;
    let query = serde_urlencoded::to_string(filter_params(&state.query)).unwrap_or_default();
    let full_url = format!("{}?{}", uri_without_query, query);

    let mut request_builder = http::Request::builder()
        .method(http::Method::from_str(&state.method.to_string()).unwrap_or_default())
        .uri(http::Uri::from_str(&full_url).unwrap_or_default())
        // TODO move to conditional auto-generated header, keeping for now
        .header(http::header::ACCEPT, HeaderValue::from_static("*/*"));

    for (header_name, header_value) in filter_params(&state.headers) {
        request_builder = request_builder.header(header_name, header_value);
    }
    let request = request_builder
        .body(state.body.clone().into_bytes())
        .unwrap();

    // Clone request so easier to pass it to another thread
    let response_store = state.response.clone();
    let auth = state.auth.clone();
    let mut request = request.clone();
    //
    // let runner = move || {
    //     auth.apply(&mut request);
    //     log::info!("{:?}", request);
    //     // let response_store = response.clone();
    //     crate::http::execute(request, move |result| match result {
    //         Ok(resp) => {
    //             *response_store.lock().unwrap() = Some(resp);
    //         }
    //         Err(resp) => match resp {
    //             HttpError::Unknown(err) => {
    //                 *response_store.lock().unwrap() = Some(HttpResponse {
    //                     body_raw: err,
    //                     ..Default::default()
    //                 })
    //             }
    //         },
    //     });
    // };
    //
    let response_store = state.response.clone();
    runtime_handle.spawn_future(async move {
        auth.apply(&mut request);
        log::info!("sending request");

        // thread::spawn doesn't work on web, so we just run the auth
        // signing on main thread which isn't slow in any means, its just
        // I didn't wanna do it
        let result = crate::http::execute_new(request).await;

        let response = match result {
            Ok(response) => response,
            // build empty response for now
            // TODO: map status codes and errors
            Err(err) => HttpResponse {
                body_raw: err.to_string(),
                ..Default::default()
            },
        };

        *response_store.lock().unwrap() = Some(response);
    });
}

fn filter_params(params: &[Param]) -> Vec<(String, String)> {
    params
        .iter()
        .filter(|p| p.enabled && !p.key.is_empty() && !p.value.is_empty())
        .map(|p| (p.key.clone(), p.value.clone()))
        .collect::<Vec<(String, String)>>()
}
