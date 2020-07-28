use crate::{
    error::Error, markdown_manager::MarkdownManager,
    servirtium_configuration::ServirtiumConfiguration,
};
use body::Bytes;
use hyper::{
    body,
    header::{HeaderName, HeaderValue},
    service::{make_service_fn, service_fn},
    Body, HeaderMap, Request, Response, Server,
};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    convert::Infallible,
    net::SocketAddr,
    path::Path,
    sync::{self, Mutex, MutexGuard},
    thread,
};
use sync::{Arc, Once};
use tokio::runtime::Runtime;

static INITIALIZE_SERVIRTIUM: Once = Once::new();

lazy_static! {
    static ref TEST_LOCK: Mutex<()> = Mutex::new(());
    static ref SERVIRTIUM_INSTANCE: Mutex<ServirtiumServer> = Mutex::new(ServirtiumServer::new());
}

pub fn prepare_for_test(
    configuration: ServirtiumConfiguration,
) -> Result<MutexGuard<'static, ()>, Error> {
    let test_lock = TEST_LOCK.lock()?;
    let mut server_lock = SERVIRTIUM_INSTANCE.lock()?;
    server_lock.start();
    server_lock.configuration = Some(Arc::new(configuration));

    Ok(test_lock)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ServirtiumMode {
    Playback,
    Record,
}

#[derive(Debug)]
pub struct ServirtiumServer {
    configuration: Option<Arc<ServirtiumConfiguration>>,
}

impl ServirtiumServer {
    fn new() -> Self {
        ServirtiumServer {
            configuration: None,
        }
    }

    fn start(&mut self) {
        INITIALIZE_SERVIRTIUM.call_once(|| {
            thread::spawn(|| {
                Runtime::new().unwrap().block_on(async {
                    let addr = SocketAddr::from(([127, 0, 0, 1], 61417));

                    let server = Server::bind(&addr).serve(make_service_fn(|_| async {
                        Ok::<_, Infallible>(service_fn(Self::handle_request))
                    }));

                    if let Err(e) = server.await {
                        eprintln!("Servirtium Server error: {}", e);
                    }
                });
            });
        });
    }

    async fn handle_request(mut request: Request<Body>) -> Result<Response<Body>, Infallible> {
        let servirtium_config = SERVIRTIUM_INSTANCE
            .lock()
            .unwrap()
            .configuration
            .clone()
            .unwrap();

        let result = match servirtium_config.interaction_mode() {
            ServirtiumMode::Playback => Self::handle_playback(servirtium_config.record_path()),
            ServirtiumMode::Record => match servirtium_config.domain_name() {
                Some(domain_name) => {
                    Self::handle_record(&mut request, domain_name, servirtium_config.record_path())
                        .await
                }
                None => Err(Error::NotConfigured),
            },
        };

        match result {
            Ok(response) => Ok(response),
            Err(error) => {
                eprintln!("An error occured: {}", error);
                let bytes = Bytes::from(error.to_string());
                Ok(Response::builder().status(500).body(bytes.into()).unwrap())
            }
        }
    }

    fn handle_playback<P: AsRef<Path>>(record_path: P) -> Result<Response<Body>, Error> {
        let playback_data = MarkdownManager::load_playback_file(record_path.as_ref())?;
        let mut response_builder = Response::builder();

        if let Some(headers_mut) = response_builder.headers_mut() {
            Self::put_headers(headers_mut, Self::filter_headers(&playback_data.headers))?;
        }

        Ok(response_builder.body(playback_data.response_body.into())?)
    }

    async fn handle_record<S: AsRef<str>, P: AsRef<Path>>(
        mut request: &mut Request<Body>,
        domain_name: S,
        record_path: P,
    ) -> Result<Response<Body>, Error> {
        let request_data = Self::read_request_data(&mut request).await?;
        let response_data = Self::forward_request(domain_name, &request_data).await?;

        MarkdownManager::save_markdown(record_path, &request_data, &response_data)?;

        let mut response_builder = Response::builder().status(response_data.status_code);

        if let Some(header_map) = response_builder.headers_mut() {
            Self::put_headers(header_map, &response_data.headers)?;
        }

        Ok(response_builder.body(response_data.body.into())?)
    }

    async fn forward_request<S: AsRef<str>>(
        domain_name: S,
        request_data: &RequestData,
    ) -> Result<ResponseData, Error> {
        let url = format!("{}{}", domain_name.as_ref(), request_data.uri);

        let response = reqwest::get(&url).await?;
        let status_code = response.status().as_u16();
        let headers = Self::extract_headers(response.headers());

        let body = response.bytes().await?;

        Ok(ResponseData {
            status_code,
            body,
            headers,
        })
    }

    async fn read_request_data(request: &mut Request<Body>) -> Result<RequestData, Error> {
        let method = request.method().to_string();
        let uri = request.uri().to_string();
        let headers = Self::extract_headers(request.headers());

        let body = body::to_bytes(request.body_mut())
            .await
            .map_err(|_| Error::InvalidBody)?;

        Ok(RequestData {
            method,
            uri,
            headers,
            body,
        })
    }

    fn put_headers<'a, I: IntoIterator<Item = (&'a String, &'a String)>>(
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

    fn extract_headers(header_map: &HeaderMap) -> HashMap<String, String> {
        // it currently ignores header values with opaque characters
        header_map
            .iter()
            .map(|(k, v)| (String::from(k.as_str()), v.to_str()))
            .filter_map(|(key, value)| value.ok().map(|v| (key, String::from(v))))
            .collect::<HashMap<_, _>>()
    }

    fn filter_headers<'a>(
        headers: &'a HashMap<String, String>,
    ) -> impl Iterator<Item = (&'a String, &'a String)> + 'a {
        headers
            .iter()
            // Transfer-Encoding: chunked shouldn't be included in local tests because all the data is
            // written immediately and reqwest panics because of that
            .filter(|(key, value)| *key != "Transfer-Encoding" || *value != "chunked")
    }
}

#[derive(Debug)]
pub struct RequestData {
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub method: String,
    pub body: Bytes,
}

#[derive(Debug)]
pub struct ResponseData {
    pub headers: HashMap<String, String>,
    pub body: Bytes,
    pub status_code: u16,
}
