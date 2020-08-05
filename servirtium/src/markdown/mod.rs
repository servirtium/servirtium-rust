pub mod error;

use crate::{interaction_manager::InteractionManager, InteractionData, RequestData, ResponseData};
use error::{
    Error, MarkdownsBodyDifference, MarkdownsDifferenceLocation, MarkdownsDifferenceType,
    MarkdownsHeaderDifference,
};
use fs::File;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Write,
    path::PathBuf,
};

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

    fn check_headers(
        lhs: &HashMap<String, String>,
        rhs: &HashMap<String, String>,
    ) -> Option<MarkdownsHeaderDifference> {
        let left_keys = lhs.keys().collect::<HashSet<_>>();
        let right_keys = rhs.keys().collect::<HashSet<_>>();
        if let Some(&diff) = left_keys.difference(&right_keys).next() {
            return Some(MarkdownsHeaderDifference {
                header_name: diff.clone(),
                old_header_value: lhs.get(diff).cloned(),
                new_header_value: rhs.get(diff).cloned(),
            });
        }

        for key in left_keys {
            let old_value = lhs.get(key).unwrap().trim();
            let new_value = rhs.get(key).unwrap().trim();

            if old_value != new_value {
                return Some(MarkdownsHeaderDifference {
                    header_name: key.clone(),
                    old_header_value: Some(old_value.into()),
                    new_header_value: Some(new_value.into()),
                });
            }
        }

        None
    }

    fn find_difference(old_body: &str, new_body: &str) -> Option<MarkdownsBodyDifference> {
        let mut line = 1;
        let mut column = 0;
        for (index, (left, right)) in old_body.chars().zip(new_body.chars()).enumerate() {
            if left == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }

            if left != right {
                return Some(MarkdownsBodyDifference {
                    line,
                    column,
                    old_context: Self::get_context(old_body, index).into(),
                    new_context: Self::get_context(new_body, index).into(),
                });
            }
        }

        None
    }

    fn get_context(body: &str, index: usize) -> &str {
        const RADIUS: usize = 10;

        let left_bound = if index >= RADIUS { index - RADIUS } else { 0 };

        let right_bound = if index + RADIUS < body.len() {
            index + RADIUS
        } else {
            body.len() - 1
        };

        &body[left_bound..right_bound]
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

            let mut header_names = interaction.request_data.headers.keys().collect::<Vec<_>>();
            header_names.sort();
            for header_name in header_names {
                write!(
                    file,
                    "{}: {}\r\n",
                    header_name,
                    interaction.request_data.headers.get(header_name).unwrap()
                )?;
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

            let mut header_names = interaction.response_data.headers.keys().collect::<Vec<_>>();
            header_names.sort();
            for header_name in header_names {
                writeln!(
                    file,
                    "{}: {}",
                    header_name,
                    interaction.response_data.headers.get(header_name).unwrap()
                )?;
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
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let markdown_data = self.load_interactions()?;

        for (interaction_data, markdown_data) in interactions.iter().zip(markdown_data.iter()) {
            let markdown_request_body =
                markdown_data.request_data.body.trim().replace("\r\n", "\n");
            let markdown_response_body = markdown_data
                .response_data
                .body
                .trim()
                .replace("\r\n", "\n");
            let new_request_body = interaction_data
                .request_data
                .body
                .trim()
                .replace("\r\n", "\n");
            let new_response_body = interaction_data
                .response_data
                .body
                .trim()
                .replace("\r\n", "\n");

            if let Some((difference, location)) =
                Self::find_difference(&markdown_request_body, &new_request_body)
                    .map(|d| (d, MarkdownsDifferenceLocation::Request))
                    .or_else(|| {
                        Self::find_difference(&markdown_response_body, &new_response_body)
                            .map(|d| (d, MarkdownsDifferenceLocation::Response))
                    })
            {
                return Err(Box::new(Error::MarkdownsDiffer(
                    MarkdownsDifferenceType::Body(difference),
                    location,
                )));
            }

            if let Some((difference, location)) = Self::check_headers(
                &markdown_data.request_data.headers,
                &interaction_data.request_data.headers,
            )
            .map(|d| (d, MarkdownsDifferenceLocation::Request))
            .or_else(|| {
                Self::check_headers(
                    &markdown_data.response_data.headers,
                    &interaction_data.response_data.headers,
                )
                .map(|d| (d, MarkdownsDifferenceLocation::Response))
            }) {
                return Err(Box::new(Error::MarkdownsDiffer(
                    MarkdownsDifferenceType::Header(difference),
                    location,
                )));
            }
        }

        Ok(())
    }
}
