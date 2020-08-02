use crate::{
    http_client::HttpClient,
    interaction_manager::InteractionManager,
    mutations::{MutationsBuilder, RequestMutation, ResponseMutation},
    ReqwestHttpClient, ServirtiumMode,
};
use std::sync::Arc;

#[derive(Debug)]
pub struct ServirtiumConfiguration {
    domain_name: Option<String>,
    interaction_mode: ServirtiumMode,
    fail_if_markdown_changed: bool,
    interaction_manager: Arc<dyn InteractionManager + Send + Sync>,
    http_client: Option<Arc<dyn HttpClient + Send + Sync>>,
    record_request_mutations: Vec<RequestMutation>,
    record_response_mutations: Vec<ResponseMutation>,
    playback_response_mutations: Vec<ResponseMutation>,
}

impl ServirtiumConfiguration {
    pub fn new(
        mode: ServirtiumMode,
        interaction_manager: Box<dyn InteractionManager + Send + Sync>,
    ) -> Self {
        Self {
            interaction_mode: mode,
            domain_name: None,
            fail_if_markdown_changed: false,
            interaction_manager: interaction_manager.into(),
            http_client: None,
            record_request_mutations: Vec::new(),
            playback_response_mutations: Vec::new(),
            record_response_mutations: Vec::new(),
        }
    }

    pub fn set_fail_if_markdown_changed(&mut self, value: bool) {
        self.fail_if_markdown_changed = value;
    }

    pub fn fail_if_markdown_changed(&self) -> bool {
        self.fail_if_markdown_changed
    }

    pub fn set_domain_name<S: Into<String>>(&mut self, domain_name: S) {
        self.domain_name = Some(domain_name.into());
    }

    pub fn domain_name(&self) -> Option<&String> {
        self.domain_name.as_ref()
    }

    pub fn interaction_mode(&self) -> ServirtiumMode {
        self.interaction_mode
    }

    pub fn interaction_manager(&self) -> Arc<dyn InteractionManager + Send + Sync> {
        self.interaction_manager.clone()
    }

    pub fn http_client(&self) -> Arc<dyn HttpClient + Send + Sync> {
        self.http_client
            .clone()
            .unwrap_or_else(|| Arc::new(ReqwestHttpClient::new()))
    }

    pub fn set_http_client(&mut self, http_client: Arc<dyn HttpClient + Send + Sync>) {
        self.http_client = Some(http_client);
    }

    pub fn add_record_request_mutations<
        F: FnOnce(&mut MutationsBuilder) -> &mut MutationsBuilder,
    >(
        &mut self,
        func: F,
    ) {
        let mut mutations = MutationsBuilder::new();
        let _ = func(&mut mutations);
        self.record_request_mutations
            .extend(mutations.into_request_mutations());
    }

    pub fn add_record_response_mutations<
        F: FnOnce(&mut MutationsBuilder) -> &mut MutationsBuilder,
    >(
        &mut self,
        func: F,
    ) {
        let mut mutations = MutationsBuilder::new();
        let _ = func(&mut mutations);
        self.record_response_mutations
            .extend(mutations.into_response_mutations());
    }

    pub fn add_playback_response_mutations<
        F: FnOnce(&mut MutationsBuilder) -> &mut MutationsBuilder,
    >(
        &mut self,
        func: F,
    ) {
        let mut mutations = MutationsBuilder::new();
        let _ = func(&mut mutations);
        self.playback_response_mutations
            .extend(mutations.into_response_mutations());
    }

    pub fn record_request_mutations(&self) -> &[RequestMutation] {
        &self.record_request_mutations
    }

    pub fn record_response_mutations(&self) -> &[ResponseMutation] {
        &self.record_response_mutations
    }

    pub fn playback_response_mutations(&self) -> &[ResponseMutation] {
        &self.playback_response_mutations
    }
}
