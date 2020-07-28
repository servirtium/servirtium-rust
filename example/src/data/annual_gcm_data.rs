use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct AnnualData {
    pub double: f64,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnnualGcmDatum {
    pub gcm: String,
    pub variable: String,
    pub from_year: String,
    pub to_year: String,
    pub annual_data: AnnualData,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename = "list")]
pub struct AnnualGcmData {
    #[serde(rename = "domain.web.AnnualGcmDatum")]
    pub results: Option<Vec<AnnualGcmDatum>>,
}
