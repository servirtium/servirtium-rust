use super::HeadersMutation;
use regex::Regex;
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

#[derive(Debug)]
pub struct RemoveHeadersRegexMutation {
    patterns: Vec<Regex>,
}

impl RemoveHeadersRegexMutation {
    pub fn new<I: IntoIterator<Item = Regex>>(patterns: I) -> Self {
        Self {
            patterns: patterns.into_iter().collect(),
        }
    }
}

impl HeadersMutation for RemoveHeadersRegexMutation {
    fn mutate(&self, headers: &mut HashMap<String, String>) {
        let mut headers_to_remove = Vec::new();
        for regex in &self.patterns {
            for header_name in headers.keys() {
                if regex.is_match(header_name) {
                    headers_to_remove.push(header_name.clone());
                }
            }
        }

        for header in headers_to_remove {
            headers.remove(&header);
        }
    }
}
