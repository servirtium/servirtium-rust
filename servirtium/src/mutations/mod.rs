mod add_header_mutation;
mod remove_headers_mutation;

use crate::InteractionData;
use add_header_mutation::AddHeaderMutation;
use remove_headers_mutation::RemoveHeadersMutation;
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

#[derive(Debug, Copy, Clone)]
enum InteractionDataType {
    Request,
    Response,
}

#[derive(Debug)]
pub struct Mutation {
    mutation_type: MutationType,
    data_type: InteractionDataType,
}

impl Mutation {
    fn from_headers_mutation<HM: HeadersMutation + Send + Sync + 'static>(
        mutation: HM,
        data_type: InteractionDataType,
    ) -> Self {
        Self {
            mutation_type: MutationType::Headers(Box::new(mutation)),
            data_type,
        }
    }

    fn from_body_mutation<BM: BodyMutation + Send + Sync + 'static>(
        mutation: BM,
        data_type: InteractionDataType,
    ) -> Self {
        Self {
            mutation_type: MutationType::Body(Box::new(mutation)),
            data_type,
        }
    }

    pub fn mutate(&self, interaction_data: &mut InteractionData) {
        match &self.mutation_type {
            MutationType::Headers(hm) => {
                let headers = match self.data_type {
                    InteractionDataType::Request => &mut interaction_data.request_data.headers,
                    InteractionDataType::Response => &mut interaction_data.response_data.headers,
                };

                hm.mutate(headers);
            }
            MutationType::Body(bm) => {
                let body = match self.data_type {
                    InteractionDataType::Request => &mut interaction_data.request_data.body,
                    InteractionDataType::Response => &mut interaction_data.response_data.body,
                };

                bm.mutate(body);
            }
        }
    }
}

pub struct Mutations {
    mutations: Vec<Mutation>,
}

impl Mutations {
    pub fn new() -> Self {
        Self {
            mutations: Vec::new(),
        }
    }

    pub fn request<F: Fn(MutationBuilder) -> MutationBuilder>(self, func: F) -> Mutations {
        let mutation_builder = MutationBuilder::new(self, InteractionDataType::Request);
        let mutation_builder = func(mutation_builder);
        mutation_builder.mutations
    }

    pub fn response<F: Fn(MutationBuilder) -> MutationBuilder>(self, func: F) -> Mutations {
        let mutation_builder = MutationBuilder::new(self, InteractionDataType::Response);
        let mutation_builder = func(mutation_builder);
        mutation_builder.mutations
    }

    pub fn into_vec(self) -> Vec<Mutation> {
        self.mutations
    }
}

pub struct MutationBuilder {
    data_type: InteractionDataType,
    mutations: Mutations,
}

impl MutationBuilder {
    fn new(mutations: Mutations, data_type: InteractionDataType) -> Self {
        Self {
            data_type,
            mutations,
        }
    }

    pub fn remove_headers<S: Into<String>, I: IntoIterator<Item = S>>(self, headers: I) -> Self {
        self.add_headers_mutation(RemoveHeadersMutation::new(headers))
    }

    pub fn add_header<S1: Into<String>, S2: Into<String>>(
        self,
        header_name: S1,
        header_value: S2,
    ) -> Self {
        self.add_headers_mutation(AddHeaderMutation::new(header_name, header_value))
    }

    pub fn add_headers_mutation<T: HeadersMutation + Send + Sync + 'static>(
        mut self,
        mutation: T,
    ) -> Self {
        self.mutations
            .mutations
            .push(Mutation::from_headers_mutation(mutation, self.data_type));
        self
    }

    pub fn add_body_mutation<T: BodyMutation + Send + Sync + 'static>(
        mut self,
        mutation: T,
    ) -> Self {
        self.mutations
            .mutations
            .push(Mutation::from_body_mutation(mutation, self.data_type));
        self
    }
}

impl Default for Mutations {
    fn default() -> Self {
        Self::new()
    }
}

// pub struct Mutations {
//     mutations: Vec<Box<dyn Mutation + Send + Sync>>,
// }

// impl Mutations {
//     pub fn new() -> Self {
//         Self {
//             mutations: Vec::new(),
//         }
//     }

//     pub fn remove_request_headers<S: Into<String>, I: IntoIterator<Item = S>>(
//         mut self,
//         headers: I,
//     ) -> Self {
//         self.mutations.push(Box::new(RemoveHeadersMutation::new(
//             headers,
//             InteractionDataType::Request,
//         )));
//         self
//     }

//     pub fn remove_response_headers<S: Into<String>, I: IntoIterator<Item = S>>(
//         mut self,
//         headers: I,
//     ) -> Self {
//         self.mutations.push(Box::new(RemoveHeadersMutation::new(
//             headers,
//             InteractionDataType::Response,
//         )));
//         self
//     }

//     pub fn into_vec(self) -> Vec<Box<dyn Mutation + Send + Sync>> {
//         self.mutations
//     }
// }

// impl Default for Mutations {
//     fn default() -> Self {
//         Self::new()
//     }
// }
