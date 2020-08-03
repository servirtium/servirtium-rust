use super::BodyMutation;
use regex::Regex;

#[derive(Debug)]
pub struct BodyReplaceMutation {
    text: String,
    substitution: String,
}

impl BodyReplaceMutation {
    pub fn new<S1: Into<String>, S2: Into<String>>(text: S1, substitution: S2) -> Self {
        BodyReplaceMutation {
            text: text.into(),
            substitution: substitution.into(),
        }
    }
}

impl BodyMutation for BodyReplaceMutation {
    fn mutate(&self, body: &mut String) {
        *body = body.replace(&self.text, &self.substitution);
    }
}

#[derive(Debug)]
pub struct BodyReplaceRegexMutation {
    pattern: Regex,
    substitution: String,
}

impl BodyReplaceRegexMutation {
    pub fn new<S: Into<String>>(pattern: Regex, substitution: S) -> Self {
        BodyReplaceRegexMutation {
            pattern,
            substitution: substitution.into(),
        }
    }
}

impl BodyMutation for BodyReplaceRegexMutation {
    fn mutate(&self, body: &mut String) {
        *body = self
            .pattern
            .replace(body, self.substitution.as_str())
            .into();
    }
}
