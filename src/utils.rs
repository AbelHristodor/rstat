use url::Url;

pub fn validate_url(input: &str) -> Result<Url, url::ParseError> {
    Url::parse(input)
}