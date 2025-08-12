use bytes::Bytes;
use http::HeaderValue;
use std::sync::Mutex;

pub struct BasicCookieStore(Mutex<cookie_store::CookieStore>);

impl BasicCookieStore {
    pub fn new() -> Self {
        Self(Mutex::new(cookie_store::CookieStore::new()))
    }
}

impl reqwest::cookie::CookieStore for BasicCookieStore {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &url::Url) {
        let iter = cookie_headers.filter_map(|val| {
            std::str::from_utf8(val.as_bytes())
                .map_err(cookie::ParseError::from)
                .and_then(cookie::Cookie::parse)
                .map(|val| val.into_owned())
                .ok()
        });

        let mut store = self.0.lock().unwrap();
        store.store_response_cookies(iter, url);
    }

    fn cookies(&self, url: &url::Url) -> Option<HeaderValue> {
        let store = self.0.lock().unwrap();
        let s = store
            .get_request_values(url)
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ");

        if s.is_empty() {
            return None;
        }

        HeaderValue::from_maybe_shared(Bytes::from(s)).ok()
    }
}
