#[derive(Debug, Clone)]
pub struct ServirtiumConfiguration {
    domain_name: Option<String>,
}

impl ServirtiumConfiguration {
    pub fn new() -> Self {
        Self { domain_name: None }
    }

    pub fn set_domain_name<S: Into<String>>(&mut self, domain_name: S) {
        self.domain_name = Some(domain_name.into());
    }

    pub fn domain_name(&self) -> Option<&String> {
        self.domain_name.as_ref()
    }
}

impl Default for ServirtiumConfiguration {
    fn default() -> Self {
        Self::new()
    }
}
