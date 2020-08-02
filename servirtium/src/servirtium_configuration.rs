use crate::{
    http_client::HttpClient, interaction_manager::InteractionManager, mutations::Mutation,
    Mutations, ReqwestHttpClient, ServirtiumMode,
};
use std::sync::Arc;

#[derive(Debug)]
pub struct ServirtiumConfiguration {
    domain_name: Option<String>,
    interaction_mode: ServirtiumMode,
    fail_if_markdown_changed: bool,
    interaction_manager: Arc<dyn InteractionManager + Send + Sync>,
    http_client: Option<Arc<dyn HttpClient + Send + Sync>>,
    record_mutations: Vec<Mutation>,
    playback_mutations: Vec<Mutation>,
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
            record_mutations: Vec::new(),
            playback_mutations: Vec::new(),
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

    pub fn add_record_mutations(&mut self, mutations: Mutations) {
        self.record_mutations.extend(mutations.into_vec());
    }

    pub fn add_playback_mutations(&mut self, mutations: Mutations) {
        self.playback_mutations.extend(mutations.into_vec());
    }

    pub fn record_mutations(&self) -> &[Mutation] {
        &self.record_mutations
    }

    pub fn playback_mutations(&self) -> &[Mutation] {
        &self.playback_mutations
    }
}
