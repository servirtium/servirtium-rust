use crate::{error::Error, servirtium_configuration::ServirtiumConfiguration};
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
    sync::{self, Mutex},
    thread,
};
use sync::{Arc, Condvar, Once};
use thread::JoinHandle;
use tokio::runtime::Runtime;

static INITIALIZE_SERVIRTIUM: Once = Once::new();

lazy_static! {
    static ref SERVIRTIUM_INSTANCE: Arc<(Mutex<Option<ServirtiumServer>>, Condvar)> =
        Arc::new((Mutex::new(Some(ServirtiumServer::new())), Condvar::new()));
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
    configuration: Option<ServirtiumConfiguration>,
    join_handle: Option<JoinHandle<()>>,
    error: Option<Error>,
    interactions: Vec<InteractionData>,
    markdown_data: Option<Vec<InteractionData>>,
    interaction_number: u8,
}

impl ServirtiumServer {
    fn new() -> Self {
        ServirtiumServer {
            configuration: None,
            join_handle: None,
            error: None,
            interactions: Vec::new(),
            markdown_data: None,
            interaction_number: 0,
        }
    }

    fn instance() -> Self {
        let (mutex, condvar) = &*SERVIRTIUM_INSTANCE.clone();
        let mut mutex = condvar
            .wait_while(mutex.lock().unwrap(), |option| option.is_none())
            .unwrap();

        let instance = mutex.take().unwrap();
        condvar.notify_one();

        instance
    }

    fn release_instance(self) {
        let (mutex, condvar) = &*SERVIRTIUM_INSTANCE.clone();
        *mutex.lock().unwrap() = Some(self);
        condvar.notify_one();
    }

    pub fn before_test(configuration: ServirtiumConfiguration) {
        Self::enter_test();

        let mut server = Self::instance();
        server.start();

        server.configuration = Some(configuration);
        server.release_instance();
    }

    pub fn after_test() -> Result<(), Error> {
        let mut instance = Self::instance();
        let mut result = Ok(());
        let mut error = instance.error.take();
        let config = instance.configuration.as_ref().unwrap();
        let interaction_manager = config.interaction_manager().clone();

        if error.is_none() && config.interaction_mode() == ServirtiumMode::Record {
            if instance
                .configuration
                .as_ref()
                .unwrap()
                .fail_if_markdown_changed()
                && interaction_manager
                    .check_data_unchanged(&instance.interactions)
                    .map_err(|e| Error::MarkdownParseError(e))?
            {
                error = Some(Error::MarkdownDataChanged);
            } else {
                error = interaction_manager
                    .save_interactions(&instance.interactions)
                    .err()
                    .map(|e| Error::MarkdownParseError(e));
            }
        }

        if let Some(error) = error {
            result = Err(error);
        }

        instance.reset();
        instance.release_instance();

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
                        Ok::<_, Infallible>(service_fn(|req| async {
                            let mut instance = Self::instance();

                            let response = instance.handle_request(req).await;
                            instance.release_instance();
                            response
                        }))
                    }));

                    if let Err(e) = server.await {
                        eprintln!("Servirtium Server error: {}", e);
                    }
                });
            }));
        });
    }

    async fn handle_request(
        &mut self,
        mut request: Request<Body>,
    ) -> Result<Response<Body>, Infallible> {
        let result = match self.configuration.as_ref().unwrap().interaction_mode() {
            ServirtiumMode::Playback => self.handle_playback(),
            ServirtiumMode::Record => match self.configuration.as_ref().unwrap().domain_name() {
                Some(_) => self.handle_record(&mut request).await,
                None => Err(Error::NotConfigured),
            },
        };

        match result {
            Ok(response) => Ok(response),
            Err(error) => {
                self.error = Some(error);
                Ok(Response::builder().status(500).body(Body::empty()).unwrap())
            }
        }
    }

    fn handle_playback(&mut self) -> Result<Response<Body>, Error> {
        let interaction_manager = self
            .configuration
            .as_mut()
            .unwrap()
            .interaction_manager()
            .clone();
        if self.markdown_data.is_none() {
            self.markdown_data = Some(
                interaction_manager
                    .load_interactions()
                    .map_err(|e| Error::MarkdownParseError(e))?,
            );
        } else {
            self.interaction_number += 1;
        }

        let playback_data = &self.markdown_data.as_ref().unwrap()[self.interaction_number as usize];
        let mut response_builder = Response::builder();

        if let Some(headers_mut) = response_builder.headers_mut() {
            Self::put_headers(
                headers_mut,
                Self::filter_headers(&playback_data.response_data.headers),
            )?;
        }

        Ok(response_builder.body(playback_data.response_data.body.clone().into())?)
    }

    async fn handle_record(
        &mut self,
        mut request: &mut Request<Body>,
    ) -> Result<Response<Body>, Error> {
        let request_data = Self::read_request_data(&mut request).await?;
        let response_data = Self::forward_request(
            self.configuration.as_mut().unwrap().domain_name().unwrap(),
            &request_data,
        )
        .await?;

        let interaction_data = InteractionData {
            interaction_number: self.interaction_number,
            request_data,
            response_data,
        };

        let mut response_builder =
            Response::builder().status(interaction_data.response_data.status_code);

        if let Some(header_map) = response_builder.headers_mut() {
            Self::put_headers(header_map, &interaction_data.response_data.headers)?;
        }
        let body = interaction_data.response_data.body.clone();

        self.interactions.push(interaction_data);

        Ok(response_builder.body(body.into())?)
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

    fn reset(&mut self) {
        self.interactions.clear();
        self.interaction_number = 0;
        self.markdown_data = None;
        self.error = None;
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

#[derive(Debug, Clone)]
pub struct InteractionData {
    pub interaction_number: u8,
    pub request_data: RequestData,
    pub response_data: ResponseData,
}

#[derive(Debug, Clone)]
pub struct RequestData {
    pub uri: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct ResponseData {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}
