pub mod error;

use crate::{interaction_manager::InteractionManager, InteractionData, RequestData, ResponseData};
use error::Error;
use fs::File;
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, fs, io::Write, path::PathBuf};

lazy_static! {
    static ref HEADER_REGEX: Regex =
        Regex::new(r"(?m)(?P<header_key>[a-zA-Z\-]+): (?P<header_value>.*?)$").unwrap();
    static ref MARKDOWN_REGEX: Regex = Regex::new(
        "(?ms)\
            \\#\\# Interaction (?P<interaction_number>[0-9]+): (?P<http_method>[A-Z]+) (?P<uri>[^ ]*)\
            \\#\\#\\# Request headers recorded for playback.*?\
            ```\\s*(?P<request_headers_part>.*?)\\s*```.*?\
            \\#\\#\\# Request body recorded for playback.*?\
            ```\\s*(?P<request_body_part>.*?)\\s*```.*?\
            \\#\\#\\# Response headers recorded for playback.*?\
            ```\\s*(?P<response_headers_part>.*?)\\s*```.*?\
            \\#\\#\\# Response body recorded for playback \\((?P<status_code>[0-9]+)[^)]*\\).*?\
            ```\\s*(?P<response_body_part>.*?)\\s*```"
    )
    .unwrap();
}

#[derive(Debug)]
pub struct MarkdownInteractionManager {
    markdown_path: PathBuf,
}

impl MarkdownInteractionManager {
    pub fn new<P: Into<PathBuf>>(markdown_path: P) -> Self {
        Self {
            markdown_path: markdown_path.into(),
        }
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

impl InteractionManager for MarkdownInteractionManager {
    fn load_interactions(
        &self,
    ) -> Result<Vec<InteractionData>, Box<dyn std::error::Error + Send + Sync>> {
        let file_contents = fs::read_to_string(&self.markdown_path)?;
        let mut data = Vec::new();

        for captures in MARKDOWN_REGEX.captures_iter(&file_contents) {
            let uri = &captures["uri"];
            let interaction_number: u8 = captures["interaction_number"]
                .parse()
                .map_err(|_| Error::InvalidInteractionNumber)?;
            let request_headers_part = &captures["request_headers_part"];
            let request_body_part = &captures["request_body_part"];
            let status_code = captures["status_code"]
                .parse()
                .map_err(|_| Error::InvalidStatusCode)?;
            let method = &captures["http_method"];
            let response_headers_part = &captures["response_headers_part"];
            let response_body_part = &captures["response_body_part"];

            let response_headers = Self::parse_headers(response_headers_part);
            let request_headers = Self::parse_headers(request_headers_part);

            data.push(InteractionData {
                interaction_number,
                request_data: RequestData {
                    body: request_body_part.into(),
                    method: method.into(),
                    headers: request_headers,
                    uri: uri.into(),
                },
                response_data: ResponseData {
                    status_code,
                    headers: response_headers,
                    body: response_body_part.into(),
                },
            });
        }

        if data.is_empty() {
            Err(Box::new(Error::InvalidMarkdownFormat))
        } else {
            Ok(data)
        }
    }

    fn save_interactions(
        &self,
        interactions: &[InteractionData],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut file = File::create(&self.markdown_path)?;
        for interaction in interactions.iter() {
            write!(
                file,
                "## Interaction {}: {} {}\r\n\r\n",
                interaction.interaction_number,
                interaction.request_data.method,
                interaction.request_data.uri
            )?;
            write!(
                file,
                "### Request headers recorded for playback:\r\n\r\n```\r\n"
            )?;
            for (key, value) in &interaction.request_data.headers {
                write!(file, "{}: {}\r\n", key, value)?;
            }
            write!(file, "```\r\n\r\n")?;

            write!(
                file,
                "### Request body recorded for playback ():\r\n\r\n```\r\n{}\r\n```\r\n\r\n",
                &interaction.request_data.body,
            )?;
            write!(
                file,
                "### Response headers recorded for playback:\r\n\r\n```\r\n"
            )?;
            for (key, value) in &interaction.response_data.headers {
                writeln!(file, "{}: {}", key, value)?;
            }
            write!(file, "```\r\n\r\n")?;
            write!(
                file,
                "### Response body recorded for playback ({}: {}):\r\n\r\n```\r\n{}\r\n```\r\n\r\n",
                interaction.response_data.status_code,
                interaction
                    .response_data
                    .headers
                    .get("content-type")
                    .unwrap_or(&String::from("")),
                &interaction.response_data.body
            )?;
        }

        Ok(())
    }

    fn check_data_unchanged(
        &self,
        interactions: &[InteractionData],
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let markdown_data = self.load_interactions()?;

        for (interaction_data, markdown_data) in interactions.iter().zip(markdown_data.iter()) {
            if markdown_data.request_data.body.trim() != interaction_data.request_data.body.trim()
                || markdown_data.response_data.body.trim()
                    != interaction_data.response_data.body.trim()
                || !Self::headers_equal(
                    &markdown_data.request_data.headers,
                    &interaction_data.request_data.headers,
                )
                || !Self::headers_equal(
                    &markdown_data.response_data.headers,
                    &interaction_data.response_data.headers,
                )
            {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
