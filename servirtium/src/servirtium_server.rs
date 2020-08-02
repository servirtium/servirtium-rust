use crate::{
    error::Error, servirtium_configuration::ServirtiumConfiguration, InteractionData, RequestData,
    ResponseData,
};
use hyper::{
    header::{HeaderName, HeaderValue},
    HeaderMap, Response, Uri,
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
        request: RequestData,
    ) -> Result<ResponseData, Error> {
        match self.configuration.as_ref().unwrap().interaction_mode() {
            ServirtiumMode::Playback => self.handle_playback(),
            ServirtiumMode::Record => self.handle_record(request).await,
        }
    }

    fn handle_playback(&mut self) -> Result<ResponseData, Error> {
        let config = self.configuration.as_mut().unwrap();
        let interaction_manager = config.interaction_manager().clone();

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

        let mut response_data = playback_data.response_data.clone();

        // mutate the response according to the configuration
        for mutation in config.playback_response_mutations() {
            mutation.mutate(&mut response_data);
        }

        let mut response_builder = Response::builder();

        if let Some(headers_mut) = response_builder.headers_mut() {
            Self::put_headers(headers_mut, Self::filter_headers(&response_data.headers))?;
        }

        Ok(response_data)
    }

    async fn handle_record(
        &mut self,
        mut request_data: RequestData,
    ) -> Result<ResponseData, Error> {
        let config = self.configuration.as_mut().unwrap();

        let http_client = config.http_client();

        Self::add_host_header(&mut request_data, config)?;

        // Mutate the request according to the configuration
        for mutation in config.record_request_mutations() {
            mutation.mutate(&mut request_data);
        }

        let mut response_data = http_client
            .make_request(
                config.domain_name().ok_or(Error::NotConfigured)?,
                &request_data,
            )
            .await?;

        // Mutate the response according to the configuration to write it to markdown
        for mutation in config.record_response_mutations() {
            mutation.mutate(&mut response_data);
        }

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

        let mut response_data = interaction_data.response_data.clone();
        self.interactions.push(interaction_data);

        // Now mutate the actual response sent to the caller
        for mutation in config.playback_response_mutations() {
            mutation.mutate(&mut response_data);
        }

        Ok(response_data)
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

    fn add_host_header(
        request_data: &mut RequestData,
        config: &ServirtiumConfiguration,
    ) -> Result<(), Error> {
        let domain_name_uri = config
            .domain_name()
            .unwrap()
            .parse::<Uri>()
            .map_err(|_| Error::InvalidDomainName)?;
        let host = domain_name_uri.host().ok_or(Error::InvalidDomainName)?;

        request_data
            .headers
            .insert(String::from("host"), String::from(host));

        Ok(())
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
