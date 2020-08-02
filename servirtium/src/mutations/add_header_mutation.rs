use super::HeadersMutation;
use std::collections::HashMap;

#[derive(Debug)]
pub struct AddHeaderMutation {
    header_name: String,
    header_value: String,
}

impl AddHeaderMutation {
    pub fn new<S1: Into<String>, S2: Into<String>>(name: S1, value: S2) -> Self {
        Self {
            header_name: name.into(),
            header_value: value.into(),
        }
    }
}

impl HeadersMutation for AddHeaderMutation {
    fn mutate(&self, headers: &mut HashMap<String, String>) {
        headers.insert(self.header_name.clone(), self.header_value.clone());
    }
}
