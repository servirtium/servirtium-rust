use crate::{interaction_manager::InteractionManager, ServirtiumMode};
use std::sync::Arc;

#[derive(Debug)]
pub struct ServirtiumConfiguration {
    domain_name: Option<String>,
    interaction_mode: ServirtiumMode,
    fail_if_markdown_changed: bool,
    interaction_manager: Arc<dyn InteractionManager + Send + Sync>,
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
        }
    }

    pub fn set_domain_name<S: Into<String>>(&mut self, domain_name: S) {
        self.domain_name = Some(domain_name.into());
    }

    pub fn set_fail_if_markdown_changed(&mut self, value: bool) {
        self.fail_if_markdown_changed = value;
    }

    pub fn fail_if_markdown_changed(&self) -> bool {
        self.fail_if_markdown_changed
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
}
