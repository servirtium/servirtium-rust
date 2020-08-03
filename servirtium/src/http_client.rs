use crate::{error::Error, util, RequestData, ResponseData};
use async_trait::async_trait;
use hyper::{body, Body, HeaderMap, Request};
use hyper_tls::HttpsConnector;
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
        domain_name: &str,
        request_data: &RequestData,
    ) -> Result<ResponseData, Error> {
        let url = format!("{}{}", domain_name, request_data.uri);
        let mut request_builder = Request::builder()
            .uri(url.as_str())
            .method(request_data.method.as_str());

        if let Some(headers_mut) = request_builder.headers_mut() {
            util::put_headers(
                headers_mut,
                request_data
                    .headers
                    .iter()
                    .filter(|(header_name, _)| header_name.as_str() != "host"),
            )?;
        }

        let request: Request<Body> = request_builder.body(request_data.body.clone().into())?;

        let client = hyper::Client::builder().build(HttpsConnector::new());

        let response = client.request(request).await?;

        let status_code = response.status().as_u16();
        let headers = Self::extract_headers(response.headers());
        let body = body::to_bytes(response.into_body()).await?;
        let body: String = String::from_utf8_lossy(&body).into();

        Ok(ResponseData {
            status_code,
            body,
            headers,
        })
    }
}

impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self::new()
    }
}
