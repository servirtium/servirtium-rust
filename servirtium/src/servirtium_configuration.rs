use crate::ServirtiumMode;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ServirtiumConfiguration {
    domain_name: Option<String>,
    interaction_mode: ServirtiumMode,
    record_path: PathBuf,
}

impl ServirtiumConfiguration {
    pub fn new<P: Into<PathBuf>>(mode: ServirtiumMode, markdown_path: P) -> Self {
        Self {
            interaction_mode: mode,
            record_path: markdown_path.into(),
            domain_name: None,
        }
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

    pub fn record_path(&self) -> &Path {
        &self.record_path
    }
}
