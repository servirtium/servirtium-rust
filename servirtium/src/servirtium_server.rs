use crate::{
    error::Error, markdown_manager::MarkdownManager,
    servirtium_configuration::ServirtiumConfiguration,
};
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
    sync::{self, Mutex},
    thread,
};
use sync::{Arc, Condvar, Once};
use thread::JoinHandle;
use tokio::runtime::Runtime;

static INITIALIZE_SERVIRTIUM: Once = Once::new();

lazy_static! {
    static ref SERVIRTIUM_INSTANCE: Mutex<ServirtiumServer> = Mutex::new(ServirtiumServer::new());
    static ref TEST_LOCK: Arc<(Mutex<bool>, Condvar)> =
        Arc::new((Mutex::new(false), Condvar::new()));
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ServirtiumMode {
    Playback,
    Record,
}

#[derive(Debug)]
pub struct ServirtiumServer {
    configuration: Option<Arc<ServirtiumConfiguration>>,
    join_handle: Option<JoinHandle<()>>,
    error: Option<Error>,
}

impl ServirtiumServer {
    fn new() -> Self {
        ServirtiumServer {
            configuration: None,
            join_handle: None,
            error: None,
        }
    }

    pub fn prepare_for_test(configuration: ServirtiumConfiguration) {
        Self::enter_test();

        let mut server_lock = SERVIRTIUM_INSTANCE.lock().unwrap();
        server_lock.start();
        server_lock.configuration = Some(Arc::new(configuration));
    }

    pub fn cleanup_after_test() -> Result<(), Error> {
        let mut result = Ok(());
        let mut instance = SERVIRTIUM_INSTANCE.lock().unwrap();
        if let Some(error) = instance.error.take() {
            result = Err(error);
        }

        Self::exit_test();

        result
    }

    fn enter_test() {
        let (lock, cond) = &*TEST_LOCK.clone();
        let mut is_test_running = cond
            .wait_while(lock.lock().unwrap(), |is_test_running| *is_test_running)
            .unwrap();
        *is_test_running = true;
    }

    fn exit_test() {
        let (lock, cond) = &*TEST_LOCK.clone();
        let mut is_test_running = lock.lock().unwrap();
        *is_test_running = false;

        cond.notify_one();
    }

    fn start(&mut self) {
        INITIALIZE_SERVIRTIUM.call_once(|| {
            self.join_handle = Some(thread::spawn(move || {
                Runtime::new().unwrap().block_on(async {
                    let addr = SocketAddr::from(([127, 0, 0, 1], 61417));

                    let server = Server::bind(&addr).serve(make_service_fn(|_| async {
                        Ok::<_, Infallible>(service_fn(Self::handle_request))
                    }));

                    if let Err(e) = server.await {
                        eprintln!("Servirtium Server error: {}", e);
                    }
                });
            }));
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
                    Self::handle_record(
                        &mut request,
                        domain_name,
                        servirtium_config.record_path(),
                        servirtium_config.fail_if_markdown_changed(),
                    )
                    .await
                }
                None => Err(Error::NotConfigured),
            },
        };

        match result {
            Ok(response) => Ok(response),
            Err(error) => {
                let mut instance = SERVIRTIUM_INSTANCE.lock().unwrap();
                instance.error = Some(error);
                Ok(Response::builder().status(500).body(Body::empty()).unwrap())
            }
        }
    }

    fn handle_playback<P: AsRef<Path>>(record_path: P) -> Result<Response<Body>, Error> {
        let playback_data = MarkdownManager::load_markdown(record_path.as_ref())?;
        let mut response_builder = Response::builder();

        if let Some(headers_mut) = response_builder.headers_mut() {
            Self::put_headers(
                headers_mut,
                Self::filter_headers(&playback_data.response_headers),
            )?;
        }

        Ok(response_builder.body(playback_data.response_body.into())?)
    }

    async fn handle_record<S: AsRef<str>, P: AsRef<Path>>(
        mut request: &mut Request<Body>,
        domain_name: S,
        record_path: P,
        fail_if_markdown_changed: bool,
    ) -> Result<Response<Body>, Error> {
        let request_data = Self::read_request_data(&mut request).await?;
        let response_data = Self::forward_request(domain_name, &request_data).await?;

        if fail_if_markdown_changed
            && !MarkdownManager::check_markdown_data_unchanged(
                &record_path,
                &request_data,
                &response_data,
            )?
        {
            return Err(Error::MarkdownDataChanged);
        }

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
            body: String::from_utf8_lossy(&body).into(),
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
            body: String::from_utf8_lossy(&body).into(),
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

impl Drop for ServirtiumServer {
    fn drop(&mut self) {
        if let Some(join_handle) = self.join_handle.take() {
            join_handle
                .join()
                .expect("Couldn't gracefully shutdown the Servirtium server thread");
        }
    }
}

#[derive(Debug)]
pub struct RequestData {
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub method: String,
    pub body: String,
}

#[derive(Debug)]
pub struct ResponseData {
    pub headers: HashMap<String, String>,
    pub body: String,
    pub status_code: u16,
}
