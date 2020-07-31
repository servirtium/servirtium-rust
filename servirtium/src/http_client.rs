use crate::{
    error::Error,
    servirtium_server::{RequestData, ResponseData},
};
use async_trait::async_trait;
use hyper::HeaderMap;
use std::{collections::HashMap, fmt::Debug};

#[async_trait]
pub trait HttpClient: Debug {
    async fn make_request(
        &self,
        url: &str,
        request_data: &RequestData,
    ) -> Result<ResponseData, Error>;
}

#[derive(Debug)]
pub struct ReqwestHttpClient {}

impl ReqwestHttpClient {
    pub fn new() -> Self {
        Self {}
    }

    fn extract_headers(header_map: &HeaderMap) -> HashMap<String, String> {
        // it currently ignores header values with opaque characters
        header_map
            .iter()
            .map(|(k, v)| (String::from(k.as_str()), v.to_str()))
            .filter_map(|(key, value)| value.ok().map(|v| (key, String::from(v))))
            .collect::<HashMap<_, _>>()
    }
}

#[async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn make_request(
        &self,
        url: &str,
        request_data: &RequestData,
    ) -> Result<ResponseData, Error> {
        let url = format!("{}{}", &url, request_data.uri);

        let response = reqwest::get(&url).await?;
        let status_code = response.status().as_u16();
        let headers = Self::extract_headers(response.headers());

        let body = response.bytes().await?;

        Ok(ResponseData {
            status_code,
            body: String::from_utf8_lossy(&body).into(),
            headers,
        })
    }
}
