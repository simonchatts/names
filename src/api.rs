//! Wrap the API calls to get gender/country info

use std::collections::HashMap;

use reqwasm::http::{Request, Response};
use serde::Deserialize;

use crate::common::*;

/// Errors from an API request.
#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("API request failed: {0}")]
    Reqwasm(#[from] reqwasm::Error),
    #[error("Daily API limit exceeded - try again tomorrow")]
    LimitExceeded,
    #[error("Server returned error code {0} ({1})")]
    ServerError(u16, String),
}

/// Generic type of an API result: either an [ApiError], or a mapping from first
/// name to a result type.
pub type ApiResult<T> = Result<HashMap<Name, T>, ApiError>;

/// Fire off a bulk gender request.
pub async fn get_genders(names: &[Name]) -> ApiResult<GenderResult> {
    Ok(fetch("genderize", names)
        .await?
        .json::<RawGenderResults>()
        .await?
        .0
        .into_iter()
        .map(|r| {
            (
                r.name,
                GenderResult {
                    gender: r.gender,
                    probability: r.probability,
                    count: r.count,
                },
            )
        })
        .collect())
}

/// Fire off a bulk country request.
pub async fn get_countries(names: &[Name]) -> ApiResult<Vec<CountryResult>> {
    Ok(fetch("nationalize", names)
        .await?
        .json::<RawCountryResults>()
        .await?
        .0
        .into_iter()
        .map(|r| (r.name, r.country))
        .collect())
}

/// Internal helper function: create an HTTP request, fire it off, and deal with
/// the most common error cases.
async fn fetch(domain: &str, names: &[Name]) -> Result<Response, ApiError> {
    let url = format!("https://api.{}.io/{}", domain, fmt_params(names));
    let response = Request::get(&url).send().await?;

    // A successful reponse does not mean an HTTP 200, so turn an unhelpful
    // server response into an error if appropriate, taking extra care for the
    // most likely error case the user might need help interpreting.
    let status = response.status();
    if status == 429 {
        Err(ApiError::LimitExceeded)
    } else if status != 200 {
        Err(ApiError::ServerError(status, response.status_text()))
    } else {
        Ok(response)
    }
}

/// Internal helper function: format the parameters. Do it manually, rather than
/// a crate, since the keys are unencoded, but the values are, and the popular
/// crates don't have a way of doing that which is simpler than just spelling it
/// our ourselves here.
fn fmt_params(names: &[Name]) -> String {
    let mut params = String::default();
    let mut first_param = true;
    for name in names {
        let sep = if first_param {
            first_param = false;
            '?'
        } else {
            '&'
        };
        params.push_str(&format!("{}name[]={}", sep, urlencoding::encode(name)));
    }
    params
}

//////////////////////////////////////////////////////////////////////////////

/// Gender result for one [Name]
#[derive(Debug, Clone)]
pub struct GenderResult {
    pub gender: Option<Gender>,
    pub probability: f32,
    pub count: u32,
}

/// Gender
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Female,
    Male,
}

//////////////////////////////////////////////////////////////////////////////

/// Single country result for one [Name]
#[derive(Debug, Clone, Deserialize)]
pub struct CountryResult {
    #[serde(rename = "country_id")]
    pub country: String,
    pub probability: f32,
}

//////////////////////////////////////////////////////////////////////////////
//
// Internals (raw API representations)

/// Direct representation of a gender API result set.
#[derive(Deserialize, Debug)]
pub struct RawGenderResults(pub Vec<RawGenderResult>);

/// Direct representation of a single gender API result.
#[derive(Deserialize, Debug)]
pub struct RawGenderResult {
    pub name: Name,
    pub gender: Option<Gender>,
    pub probability: f32,
    pub count: u32,
}

/// Direct representation of a country API result set.
#[derive(Deserialize, Debug)]
pub struct RawCountryResults(pub Vec<RawCountryResult>);

/// Direct representation of a single country API result.
#[derive(Deserialize, Debug)]
pub struct RawCountryResult {
    pub name: Name,
    pub country: Vec<CountryResult>,
}
