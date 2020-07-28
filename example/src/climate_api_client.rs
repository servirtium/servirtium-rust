use crate::error::Error;
use reqwest::{self};
type ReqwestClient = reqwest::blocking::Client;
use crate::data::annual_gcm_data::AnnualGcmData;

const DEFAULT_DOMAIN_NAME: &str = "http://climatedataapi.worldbank.org";

/// Builder used to build a ClimateApiClient instance
#[derive(Debug, Clone, Default)]
pub struct ClimateApiClientBuilder {
    domain_name: Option<String>,
    http_client: Option<ReqwestClient>,
}

impl ClimateApiClientBuilder {
    /// Create a new ClimateApiClientBuilder instance.
    pub fn new() -> Self {
        Self {
            domain_name: None,
            http_client: None,
        }
    }

    /// Use the given domain_name when building a ClimateApiClient instance.
    ///
    /// # Arguments
    /// `domain_name` - a domain name to use when calling the API.
    ///
    /// # Returns
    /// This builder.
    pub fn with_domain_name<T: Into<String>>(mut self, domain_name: T) -> Self {
        self.domain_name = Some(domain_name.into());
        self
    }

    /// Use the given blocking reqwest client when building a ClimateApiClient instance.
    ///
    /// # Arguments
    /// `client` - a pre-configured blocking reqwest client.
    ///
    /// # Returns
    /// This builder.
    pub fn with_http_client(mut self, client: ReqwestClient) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Consume the builder and create a ClimateApiClient instance using all of the previously configured values or
    /// their defaults.
    ///
    /// # Returns
    /// A ClimateApiClient instance.
    pub fn build(mut self) -> ClimateApiClient {
        ClimateApiClient {
            http: self.http_client.take().unwrap_or_default(),
            domain_name: self
                .domain_name
                .take()
                .unwrap_or_else(|| String::from(DEFAULT_DOMAIN_NAME)),
        }
    }
}

/// Struct that represents a World Bank Climate Data API client.
#[derive(Default, Debug, Clone)]
pub struct ClimateApiClient {
    http: ReqwestClient,
    domain_name: String,
}

impl ClimateApiClient {
    /// Create a ClimateApiClient with the default reqwest client.
    ///
    /// # Returns
    /// A ClimateApiClient.
    pub fn new() -> Self {
        ClimateApiClient {
            http: ReqwestClient::new(),
            domain_name: String::from(DEFAULT_DOMAIN_NAME),
        }
    }

    /// Gets an average annual rainfall data from WorldBank Climate Data API.
    ///
    /// # Arguments
    /// `from_year` - start of the year interval. It should be a value between 1920 and 2080 inclusive and it should be
    ///     divisible by 20.
    /// `to_year` - end of the year interval. It should be a value equal to `from_year` + 19.
    /// `country_iso` - ISO3 country code
    ///
    /// # Returns
    /// Average of all of the average annual values from all Global Circulation Models (GCM).
    pub fn get_average_annual_rainfall<T: AsRef<str>>(
        &self,
        from_year: u16,
        to_year: u16,
        country_iso: T,
    ) -> Result<f64, Error> {
        Self::check_years(from_year, to_year)?;

        let url = self.construct_get_average_annual_rainfall_url(from_year, to_year, country_iso);

        let response_text = self.http.get(&url).send()?.error_for_status()?.text()?;

        if response_text.starts_with("Invalid country code") {
            return Err(Error::NotRecognizedByClimateWeb);
        }

        let data: AnnualGcmData = quick_xml::de::from_str(&response_text)?;
        let data = data.results.unwrap_or_default();

        let (sum, count) = data.into_iter().fold((0.0, 0), |(sum, count), datum| {
            (sum + datum.annual_data.double, count + 1)
        });

        Ok(match count {
            0 => 0.0,
            _ => sum / count as f64,
        })
    }

    fn construct_get_average_annual_rainfall_url<T: AsRef<str>>(
        &self,
        from_year: u16,
        to_year: u16,
        country_iso: T,
    ) -> String {
        format!(
            "{}/climateweb/rest/v1/country/annualavg/pr/{}/{}/{}.xml",
            self.domain_name,
            from_year,
            to_year,
            country_iso.as_ref()
        )
    }

    fn check_years(from_year: u16, to_year: u16) -> Result<(), Error> {
        if from_year < 1920 || from_year > 2080 || from_year % 20 != 0 || to_year != from_year + 19
        {
            Err(Error::DateRangeNotSupported(from_year, to_year))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use crate::{error::Error, ClimateApiClient, ClimateApiClientBuilder};
    use servirtium::{servirtium_playback_test, servirtium_record_test};

    fn servirtium_configure(config: &mut servirtium::ServirtiumConfiguration) {
        config.set_domain_name("http://climatedataapi.worldbank.org");
    }

    #[test]
    fn test_averageRainfallForGreatBritainFrom1980to1999Exists_direct() {
        test_averageRainfallForGreatBritainFrom1980to1999Exists(ClimateApiClient::new());
    }

    #[servirtium_playback_test(
        "playback_data/average_Rainfall_For_Great_Britain_From_1980_to_1999_Exists.md",
        servirtium_configure
    )]
    fn test_averageRainfallForGreatBritainFrom1980to1999Exists_playback() {
        test_averageRainfallForGreatBritainFrom1980to1999Exists(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    #[servirtium_record_test(
        "playback_data/average_Rainfall_For_Great_Britain_From_1980_to_1999_Exists.md",
        servirtium_configure
    )]
    fn test_averageRainfallForGreatBritainFrom1980to1999Exists_record() {
        test_averageRainfallForGreatBritainFrom1980to1999Exists(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    fn test_averageRainfallForGreatBritainFrom1980to1999Exists(climateApi: ClimateApiClient) {
        assert_eq!(
            climateApi
                .get_average_annual_rainfall(1980, 1999, "gbr")
                .unwrap(),
            988.8454972331015
        );
    }

    #[test]
    fn test_averageRainfallForFranceFrom1980to1999Exists_direct() {
        test_averageRainfallForFranceFrom1980to1999Exists(ClimateApiClient::new());
    }

    #[servirtium_playback_test(
        "playback_data/average_Rainfall_For_France_From_1980_to_1999_Exists.md",
        servirtium_configure
    )]
    fn test_averageRainfallForFranceFrom1980to1999Exists_playback() {
        test_averageRainfallForFranceFrom1980to1999Exists(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    #[servirtium_record_test(
        "playback_data/average_Rainfall_For_France_From_1980_to_1999_Exists.md",
        servirtium_configure
    )]
    fn test_averageRainfallForFranceFrom1980to1999Exists_record() {
        test_averageRainfallForFranceFrom1980to1999Exists(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    fn test_averageRainfallForFranceFrom1980to1999Exists(climateApi: ClimateApiClient) {
        assert_eq!(
            climateApi
                .get_average_annual_rainfall(1980, 1999, "fra")
                .unwrap(),
            913.7986955122727
        );
    }

    #[test]
    fn test_averageRainfallForEgyptFrom1980to1999Exists_direct() {
        test_averageRainfallForEgyptFrom1980to1999Exists(ClimateApiClient::new());
    }

    #[servirtium_playback_test(
        "playback_data/average_Rainfall_For_Egypt_From_1980_to_1999_Exists.md",
        servirtium_configure
    )]
    fn test_averageRainfallForEgyptFrom1980to1999Exists_playback() {
        test_averageRainfallForEgyptFrom1980to1999Exists(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    #[servirtium_record_test(
        "playback_data/average_Rainfall_For_Egypt_From_1980_to_1999_Exists.md",
        servirtium_configure
    )]
    fn test_averageRainfallForEgyptFrom1980to1999Exists_record() {
        test_averageRainfallForEgyptFrom1980to1999Exists(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    fn test_averageRainfallForEgyptFrom1980to1999Exists(climateApi: ClimateApiClient) {
        assert_eq!(
            climateApi
                .get_average_annual_rainfall(1980, 1999, "egy")
                .unwrap(),
            54.58587712129825
        );
    }

    #[test]
    fn test_averageRainfallForGreatBritainFrom1985to1995DoesNotExist_direct() {
        test_averageRainfallForGreatBritainFrom1985to1995DoesNotExist(ClimateApiClient::new());
    }

    #[servirtium_playback_test(
        "playback_data/average_Rainfall_For_Great_Britain_From_1985_to_1995_Does_Not_Exist.md",
        servirtium_configure
    )]
    fn test_averageRainfallForGreatBritainFrom1985to1995DoesNotExist_playback() {
        test_averageRainfallForGreatBritainFrom1985to1995DoesNotExist(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    #[servirtium_record_test(
        "playback_data/average_Rainfall_For_Great_Britain_From_1985_to_1995_Does_Not_Exist.md",
        servirtium_configure
    )]
    fn test_averageRainfallForGreatBritainFrom1985to1995DoesNotExist_record() {
        test_averageRainfallForGreatBritainFrom1985to1995DoesNotExist(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    fn test_averageRainfallForGreatBritainFrom1985to1995DoesNotExist(climateApi: ClimateApiClient) {
        let result = climateApi.get_average_annual_rainfall(1985, 1995, "gbr");

        match result {
            Err(err) => match err {
                Error::DateRangeNotSupported(1985, 1995) => (),
                _ => panic!("The function returned a wrong error: {}", err.to_string()),
            },
            _ => panic!("The function call should return an error"),
        }
    }

    #[test]
    fn test_averageRainfallForMiddleEarthFrom1980to1999DoesNotExist_direct() {
        test_averageRainfallForMiddleEarthFrom1980to1999DoesNotExist(ClimateApiClient::new());
    }

    #[servirtium_playback_test(
        "playback_data/average_Rainfall_For_Middle_Earth_From_1980_to_1999_Does_Not_Exist.md",
        servirtium_configure
    )]
    fn test_averageRainfallForMiddleEarthFrom1980to1999DoesNotExist_playback() {
        test_averageRainfallForMiddleEarthFrom1980to1999DoesNotExist(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    #[servirtium_record_test(
        "playback_data/average_Rainfall_For_Middle_Earth_From_1980_to_1999_Does_Not_Exist.md",
        servirtium_configure
    )]
    fn test_averageRainfallForMiddleEarthFrom1980to1999DoesNotExist_record() {
        test_averageRainfallForMiddleEarthFrom1980to1999DoesNotExist(
            ClimateApiClientBuilder::new()
                .with_domain_name("http://localhost:61417")
                .build(),
        );
    }

    fn test_averageRainfallForMiddleEarthFrom1980to1999DoesNotExist(climateApi: ClimateApiClient) {
        let result = climateApi.get_average_annual_rainfall(1980, 1999, "mde");

        match result {
            Err(err) => match err {
                Error::NotRecognizedByClimateWeb => (),
                _ => panic!("The function returned a wrong error: {}", err.to_string()),
            },
            _ => panic!("The function call should return an error"),
        }
    }
}
