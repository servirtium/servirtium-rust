use crate::{
    error::Error,
    servirtium_server::{RequestData, ResponseData},
};
use fs::File;
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, fs, io::Write, path::Path};

lazy_static! {
    static ref HEADER_REGEX: Regex =
        Regex::new(r"(?m)(?P<header_key>[a-zA-Z\-]+): (?P<header_value>.*?)$").unwrap();
    static ref MARKDOWN_REGEX: Regex = Regex::new(
        "(?ms)\
            \\#\\# Interaction (?P<interaction_number>[0-9]+): (?P<http_method>[A-Z]+) \
            (?P<uri>[^ ]*)\
            \\#\\#\\# Request headers recorded for playback.*?\
            ```\\s*(?P<request_headers_part>.*?)\\s*```.*?\
            \\#\\#\\# Request body recorded for playback.*?\
            ```\\s*(?P<request_body_part>.*?)\\s*```.*?\
            \\#\\#\\# Response headers recorded for playback.*?\
            ```\\s*(?P<response_headers_part>.*?)\\s*```.*?\
            \\#\\#\\# Response body recorded for playback \\((?P<status_code>[0-9]+)[^)]*\\).*?\
            ```\\s*(?P<response_body_part>.*?)\\s*```.*?"
    )
    .unwrap();
}

#[derive(Debug, Clone)]
pub struct MarkdownData {
    pub interaction_number: u8,
    pub uri: String,
    pub method: String,
    pub request_headers: HashMap<String, String>,
    pub request_body: String,

    pub status_code: u16,
    pub response_headers: HashMap<String, String>,
    pub response_body: String,
}

pub struct MarkdownManager;

impl MarkdownManager {
    pub fn load_markdown<P: AsRef<Path>>(filename: P) -> Result<MarkdownData, Error> {
        let file_contents = fs::read_to_string(filename)?;

        let markdown_captures = MARKDOWN_REGEX
            .captures(&file_contents)
            .ok_or(Error::InvalidMarkdownFormat)?;

        let uri = &markdown_captures["uri"];
        let interaction_number = markdown_captures["interaction_number"]
            .parse()
            .map_err(|_| Error::InvalidInteractionNumber)?;
        let request_headers_part = &markdown_captures["request_headers_part"];
        let request_body_part = &markdown_captures["request_body_part"];
        let status_code = markdown_captures["status_code"]
            .parse()
            .map_err(|_| Error::InvalidStatusCode)?;
        let method = &markdown_captures["http_method"];
        let response_headers_part = &markdown_captures["response_headers_part"];
        let response_body_part = &markdown_captures["response_body_part"];

        let response_headers = Self::parse_headers(response_headers_part);
        let request_headers = Self::parse_headers(request_headers_part);

        Ok(MarkdownData {
            request_body: request_body_part.into(),
            interaction_number,
            status_code,
            method: method.into(),
            request_headers,
            response_headers,
            response_body: response_body_part.into(),
            uri: uri.into(),
        })
    }

    fn parse_headers<T: AsRef<str>>(headers_part: T) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        for capture in HEADER_REGEX.captures_iter(headers_part.as_ref()) {
            headers.insert(
                String::from(capture["header_key"].trim()),
                String::from(capture["header_value"].trim()),
            );
        }

        headers
    }

    pub fn save_markdown<P: AsRef<Path>>(
        markdown_path: P,
        request_data: &RequestData,
        response_data: &ResponseData,
    ) -> Result<(), Error> {
        let mut file = File::create(markdown_path.as_ref())?;

        write!(
            file,
            "## Interaction 0: {} {}\r\n\r\n",
            request_data.method, request_data.uri
        )?;
        write!(
            file,
            "### Request headers recorded for playback:\r\n\r\n```\r\n"
        )?;
        for (key, value) in &request_data.headers {
            write!(file, "{}: {}\r\n", key, value)?;
        }
        write!(file, "```\r\n\r\n")?;

        write!(
            file,
            "### Request body recorded for playback ():\r\n\r\n```\r\n{}\r\n```\r\n\r\n",
            &request_data.body,
        )?;
        write!(
            file,
            "### Response headers recorded for playback:\r\n\r\n```\r\n"
        )?;
        for (key, value) in &response_data.headers {
            writeln!(file, "{}: {}", key, value)?;
        }
        write!(file, "```\r\n\r\n")?;
        write!(
            file,
            "### Response body recorded for playback ({}: {}):\r\n\r\n```\r\n{}\r\n```\r\n\r\n",
            response_data.status_code,
            response_data
                .headers
                .get("content-type")
                .unwrap_or(&String::from("")),
            &response_data.body
        )?;

        Ok(())
    }

    pub fn check_markdown_data_unchanged<P: AsRef<Path>>(
        markdown_path: P,
        request_data: &RequestData,
        response_data: &ResponseData,
    ) -> Result<bool, Error> {
        let markdown_data = Self::load_markdown(markdown_path)?;

        Ok(
            markdown_data.request_body.trim() == request_data.body.trim()
                && markdown_data.response_body.trim() == response_data.body.trim()
                && Self::headers_equal(&markdown_data.request_headers, &request_data.headers)
                && Self::headers_equal(&markdown_data.response_headers, &response_data.headers),
        )
    }

    fn headers_equal(lhs: &HashMap<String, String>, rhs: &HashMap<String, String>) -> bool {
        if lhs.len() != rhs.len() {
            return false;
        }

        for (key, value) in lhs {
            match rhs.get(key) {
                Some(header) => {
                    if header.trim() != value.trim() {
                        return false;
                    }
                }
                None => return false,
            };
        }

        true
    }
}
