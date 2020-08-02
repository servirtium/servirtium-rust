use crate::error::Error;
use hyper::{
    header::{HeaderName, HeaderValue},
    HeaderMap,
};
use std::collections::HashMap;

pub fn extract_headers(header_map: &HeaderMap) -> HashMap<String, String> {
    // it currently ignores header values with opaque characters
    header_map
        .iter()
        .map(|(k, v)| (String::from(k.as_str()), v.to_str()))
        .filter_map(|(key, value)| value.ok().map(|v| (key, String::from(v))))
        .collect::<HashMap<_, _>>()
}

pub fn put_headers<'a, I: IntoIterator<Item = (&'a String, &'a String)>>(
    header_map: &mut HeaderMap<HeaderValue>,
    headers: I,
) -> Result<(), Error> {
    for (key, value) in headers {
        let header_name = HeaderName::from_lowercase(key.to_lowercase().as_bytes())?;
        let header_value = HeaderValue::from_str(value)?;
        header_map.append(header_name, header_value);
    }

    Ok(())
}
