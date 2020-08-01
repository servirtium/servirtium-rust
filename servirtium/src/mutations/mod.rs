use crate::InteractionData;
use std::fmt::Debug;

pub trait InteractionDataMutation: Debug {
    fn mutate(&self, interaction_data: &mut InteractionData);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InteractionDataType {
    Request,
    Response,
}

pub struct Mutations {
    mutations: Vec<Box<dyn InteractionDataMutation + Send + Sync>>,
}

impl Mutations {
    pub fn new() -> Self {
        Self {
            mutations: Vec::new(),
        }
    }

    pub fn remove_request_headers<S: Into<String>, I: IntoIterator<Item = S>>(
        mut self,
        headers: I,
    ) -> Self {
        self.mutations.push(Box::new(RemoveHeadersMutation::new(
            headers,
            InteractionDataType::Request,
        )));
        self
    }

    pub fn remove_response_headers<S: Into<String>, I: IntoIterator<Item = S>>(
        mut self,
        headers: I,
    ) -> Self {
        self.mutations.push(Box::new(RemoveHeadersMutation::new(
            headers,
            InteractionDataType::Response,
        )));
        self
    }

    pub fn into_vec(self) -> Vec<Box<dyn InteractionDataMutation + Send + Sync>> {
        self.mutations
    }
}

impl Default for Mutations {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct RemoveHeadersMutation {
    headers: Vec<String>,
    data_type: InteractionDataType,
}

impl RemoveHeadersMutation {
    pub fn new<S: Into<String>, I: IntoIterator<Item = S>>(
        headers: I,
        data_type: InteractionDataType,
    ) -> Self {
        Self {
            headers: headers
                .into_iter()
                .map(|e| e.into().to_lowercase())
                .collect(),
            data_type,
        }
    }
}

impl InteractionDataMutation for RemoveHeadersMutation {
    fn mutate(&self, interaction_data: &mut InteractionData) {
        match self.data_type {
            InteractionDataType::Request => {
                for header_name in &self.headers {
                    interaction_data.request_data.headers.remove(header_name);
                }
            }
            InteractionDataType::Response => {
                for header_name in &self.headers {
                    interaction_data.response_data.headers.remove(header_name);
                }
            }
        }
    }
}

// struct Add
