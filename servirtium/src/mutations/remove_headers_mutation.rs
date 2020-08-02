use super::HeadersMutation;
use std::collections::HashMap;

#[derive(Debug)]
pub struct RemoveHeadersMutation {
    headers: Vec<String>,
}

impl RemoveHeadersMutation {
    pub fn new<S: Into<String>, I: IntoIterator<Item = S>>(headers: I) -> Self {
        Self {
            headers: headers
                .into_iter()
                .map(|e| e.into().to_lowercase())
                .collect(),
        }
    }
}

impl HeadersMutation for RemoveHeadersMutation {
    fn mutate(&self, headers: &mut HashMap<String, String>) {
        for header_name in &self.headers {
            headers.remove(header_name);
        }
    }
}
