use crate::ServirtiumMode;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ServirtiumConfiguration {
    domain_name: Option<String>,
    interaction_mode: ServirtiumMode,
    record_path: PathBuf,
    fail_if_markdown_changed: bool,
}

impl ServirtiumConfiguration {
    pub fn new<P: Into<PathBuf>>(mode: ServirtiumMode, markdown_path: P) -> Self {
        Self {
            interaction_mode: mode,
            record_path: markdown_path.into(),
            domain_name: None,
            fail_if_markdown_changed: false,
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

    pub fn record_path(&self) -> &Path {
        &self.record_path
    }
}
