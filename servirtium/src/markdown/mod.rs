pub mod error;

use crate::servirtium_server::InteractionData;
use error::Error;
use fs::File;
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, fs, io::Write, path::Path};

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

pub fn load_markdown<P: AsRef<Path>>(filename: P) -> Result<Vec<MarkdownData>, Error> {
    let file_contents = fs::read_to_string(filename)?;
    let mut data = Vec::new();

    for captures in MARKDOWN_REGEX.captures_iter(&file_contents) {
        let uri = &captures["uri"];
        let interaction_number = captures["interaction_number"]
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

        let response_headers = parse_headers(response_headers_part);
        let request_headers = parse_headers(request_headers_part);

        data.push(MarkdownData {
            request_body: request_body_part.into(),
            interaction_number,
            status_code,
            method: method.into(),
            request_headers,
            response_headers,
            response_body: response_body_part.into(),
            uri: uri.into(),
        });
    }

    if data.is_empty() {
        Err(Error::InvalidMarkdownFormat)
    } else {
        Ok(data)
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

pub fn save_interactions<'a, P: AsRef<Path>, I: IntoIterator<Item = &'a InteractionData>>(
    markdown_path: P,
    interactions: I,
) -> Result<(), Error> {
    let mut file = File::create(markdown_path.as_ref())?;
    for (number, interaction) in interactions.into_iter().enumerate() {
        write!(
            file,
            "## Interaction {}: {} {}\r\n\r\n",
            number, interaction.request_data.method, interaction.request_data.uri
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

pub fn check_markdown_data_unchanged<
    'a,
    P: AsRef<Path>,
    I: IntoIterator<Item = &'a InteractionData>,
>(
    markdown_path: P,
    interactions: I,
) -> Result<bool, Error> {
    let markdown_data = load_markdown(markdown_path)?;

    for (interaction_data, markdown_data) in interactions.into_iter().zip(markdown_data.iter()) {
        if markdown_data.request_body.trim() != interaction_data.request_data.body.trim()
            || markdown_data.response_body.trim() != interaction_data.response_data.body.trim()
            || !headers_equal(
                &markdown_data.request_headers,
                &interaction_data.request_data.headers,
            )
            || !headers_equal(
                &markdown_data.response_headers,
                &interaction_data.response_data.headers,
            )
        {
            return Ok(false);
        }
    }

    Ok(true)
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
