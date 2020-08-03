mod add_header_mutation;
mod body_replace_mutation;
mod remove_headers_mutation;

use crate::{RequestData, ResponseData};
use add_header_mutation::AddHeaderMutation;
use body_replace_mutation::{BodyReplaceMutation, BodyReplaceRegexMutation};
use regex::Regex;
use remove_headers_mutation::{RemoveHeadersMutation, RemoveHeadersRegexMutation};
use std::{collections::HashMap, fmt::Debug};

pub trait BodyMutation: Debug {
    fn mutate(&self, body: &mut String);
}

pub trait HeadersMutation: Debug {
    fn mutate(&self, headers: &mut HashMap<String, String>);
}

#[derive(Debug)]
enum MutationType {
    Body(Box<dyn BodyMutation + Send + Sync>),
    Headers(Box<dyn HeadersMutation + Send + Sync>),
}

#[derive(Debug)]
pub struct RequestMutation {
    mutation_type: MutationType,
}

impl RequestMutation {
    fn from_mutation_type(mutation_type: MutationType) -> Self {
        Self { mutation_type }
    }

    pub fn mutate(&self, request_data: &mut RequestData) {
        match &self.mutation_type {
            MutationType::Headers(hm) => {
                hm.mutate(&mut request_data.headers);
            }
            MutationType::Body(bm) => {
                bm.mutate(&mut request_data.body);
            }
        }
    }
}

#[derive(Debug)]
pub struct ResponseMutation {
    mutation_type: MutationType,
}

impl ResponseMutation {
    fn from_mutation_type(mutation_type: MutationType) -> Self {
        Self { mutation_type }
    }

    pub fn mutate(&self, response_data: &mut ResponseData) {
        match &self.mutation_type {
            MutationType::Headers(hm) => {
                hm.mutate(&mut response_data.headers);
            }
            MutationType::Body(bm) => {
                bm.mutate(&mut response_data.body);
            }
        }
    }
}

pub struct MutationsBuilder {
    mutations: Vec<MutationType>,
}

impl MutationsBuilder {
    pub(crate) fn new() -> Self {
        Self {
            mutations: Vec::new(),
        }
    }

    pub fn remove_headers<S: Into<String>, I: IntoIterator<Item = S>>(
        &mut self,
        headers: I,
    ) -> &mut Self {
        self.add_headers_mutation(RemoveHeadersMutation::new(headers))
    }

    pub fn remove_headers_regex<I: IntoIterator<Item = Regex>>(
        &mut self,
        patterns: I,
    ) -> &mut Self {
        self.add_headers_mutation(RemoveHeadersRegexMutation::new(patterns))
    }

    pub fn add_header<S1: Into<String>, S2: Into<String>>(
        &mut self,
        header_name: S1,
        header_value: S2,
    ) -> &mut Self {
        self.add_headers_mutation(AddHeaderMutation::new(header_name, header_value))
    }

    pub fn body_replace<S1: Into<String>, S2: Into<String>>(
        &mut self,
        text: S1,
        replacement: S2,
    ) -> &mut Self {
        self.add_body_mutation(BodyReplaceMutation::new(text, replacement))
    }

    pub fn body_replace_regex<S: Into<String>>(
        &mut self,
        pattern: Regex,
        replacement: S,
    ) -> &mut Self {
        self.add_body_mutation(BodyReplaceRegexMutation::new(pattern, replacement))
    }

    pub fn add_headers_mutation<HM: HeadersMutation + Send + Sync + 'static>(
        &mut self,
        mutation: HM,
    ) -> &mut Self {
        self.mutations
            .push(MutationType::Headers(Box::new(mutation)));
        self
    }

    pub fn add_body_mutation<BM: BodyMutation + Send + Sync + 'static>(
        &mut self,
        mutation: BM,
    ) -> &mut Self {
        self.mutations.push(MutationType::Body(Box::new(mutation)));
        self
    }

    pub fn into_response_mutations(self) -> Vec<ResponseMutation> {
        self.mutations
            .into_iter()
            .map(ResponseMutation::from_mutation_type)
            .collect()
    }

    pub fn into_request_mutations(self) -> Vec<RequestMutation> {
        self.mutations
            .into_iter()
            .map(RequestMutation::from_mutation_type)
            .collect()
    }
}

impl Default for MutationsBuilder {
    fn default() -> Self {
        Self::new()
    }
}
