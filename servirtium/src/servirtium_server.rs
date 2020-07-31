use crate::{error::Error, servirtium_configuration::ServirtiumConfiguration};
use hyper::{
    header::{HeaderName, HeaderValue},
    HeaderMap, Response,
};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{self, Mutex},
    thread,
};
use sync::{Arc, Condvar};
use thread::JoinHandle;

lazy_static! {
    static ref SERVIRTIUM_INSTANCE: Arc<(Mutex<Option<ServirtiumServer>>, Condvar)> =
        Arc::new((Mutex::new(Some(ServirtiumServer::new())), Condvar::new()));
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ServirtiumMode {
    Playback,
    Record,
}

#[derive(Debug)]
pub struct ServirtiumServer {
    pub(crate) configuration: Option<ServirtiumConfiguration>,
    pub(crate) join_handle: Option<JoinHandle<()>>,
    error: Option<Error>,
    pub(crate) interactions: Vec<InteractionData>,
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

    pub(crate) fn instance() -> Self {
        let (mutex, condvar) = &*SERVIRTIUM_INSTANCE.clone();
        let mut mutex = condvar
            .wait_while(mutex.lock().unwrap(), |option| option.is_none())
            .unwrap();

        let instance = mutex.take().unwrap();
        condvar.notify_one();

        instance
    }

    pub(crate) fn release_instance(self) {
        let (mutex, condvar) = &*SERVIRTIUM_INSTANCE.clone();
        *mutex.lock().unwrap() = Some(self);
        condvar.notify_one();
    }

    pub(crate) async fn handle_request(
        &mut self,
        mut request: RequestData,
    ) -> Result<ResponseData, Error> {
        match self.configuration.as_ref().unwrap().interaction_mode() {
            ServirtiumMode::Playback => self.handle_playback(),
            ServirtiumMode::Record => match self.configuration.as_ref().unwrap().domain_name() {
                Some(_) => self.handle_record(&mut request).await,
                None => Err(Error::NotConfigured),
            },
        }
    }

    fn handle_playback(&mut self) -> Result<ResponseData, Error> {
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

        Ok(playback_data.response_data.clone())
    }

    async fn handle_record(
        &mut self,
        request_data: &mut RequestData,
    ) -> Result<ResponseData, Error> {
        let response_data = Self::forward_request(
            self.configuration.as_mut().unwrap().domain_name().unwrap(),
            request_data,
        )
        .await?;

        let interaction_data = InteractionData {
            interaction_number: self.interaction_number,
            request_data: request_data.clone(),
            response_data,
        };

        let mut response_builder =
            Response::builder().status(interaction_data.response_data.status_code);

        if let Some(header_map) = response_builder.headers_mut() {
            Self::put_headers(header_map, &interaction_data.response_data.headers)?;
        }

        let response_data = interaction_data.response_data.clone();

        self.interactions.push(interaction_data);

        Ok(response_data)
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

    fn filter_headers<'a>(
        headers: &'a HashMap<String, String>,
    ) -> impl Iterator<Item = (&'a String, &'a String)> + 'a {
        headers
            .iter()
            // Transfer-Encoding: chunked shouldn't be included in local tests because all the data is
            // written immediately and reqwest panics because of that
            .filter(|(key, value)| *key != "Transfer-Encoding" || *value != "chunked")
    }

    pub(crate) fn reset(&mut self) {
        self.interactions.clear();
        self.interaction_number = 0;
        self.markdown_data = None;
        self.error = None;
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
