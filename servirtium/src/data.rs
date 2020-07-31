use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct InteractionData {
    pub interaction_number: u8,
    pub request_data: RequestData,
    pub response_data: ResponseData,
}

#[derive(Debug, Clone)]
pub struct RequestData {
    pub uri: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct ResponseData {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}
