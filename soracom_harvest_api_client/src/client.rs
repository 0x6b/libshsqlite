//! Simple Soracom Harvest Data API client to get data entries and delete data entry.

use crate::{endpoint::Endpoint, error::SoracomHarvestClientError};
use chrono::{Duration, TimeZone, Utc};
use reqwest::{blocking::Client, header::USER_AGENT};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use typed_builder::TypedBuilder;

#[derive(Serialize, Deserialize, Debug)]
struct AuthRequest {
    #[serde(rename = "authKeyId")]
    pub auth_key_id: String,
    #[serde(rename = "authKey")]
    pub auth_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthResponse {
    #[serde(rename = "apiKey")]
    pub api_key: String,
    #[serde(rename = "token")]
    pub token: String,
    #[serde(rename = "userName")]
    pub user_name: Option<String>,
    #[serde(rename = "operatorId")]
    pub operator_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
struct HarvestDataResponse {
    pub data: Vec<Data>,
}

/// Single entity of Soracom Harvest Data.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Data {
    /// Epoch time of the entity.
    pub time: i64,

    /// Content type of the entity.
    #[serde(rename = "contentType")]
    pub content_type: String,

    /// Content of the entity. If value of the `content` property is a string like `{"payload": "value"}`,
    /// it could be base64-encoded data. If value of the `payload` property can be decoded as base64,
    /// and can be represented as UTF-8 string, and the decoded string has only ASCII printable characters,
    /// return `{"value": "<decoded string>"}` as the content. Otherwise return original content as is.
    pub content: String,
}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | {} | {} | {}",
            self.time,
            Utc.timestamp_millis_opt(self.time).unwrap(),
            self.content_type,
            self.content,
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Base64EncodedPayload {
    pub payload: String,
}

/// Client for Soracom Harvest Data.
///
/// Use `.builder()` to construct a new, with following methods.
///
/// - Required: `auth_key_id` and `auth_key_secret`
/// - Optional: `endpoint`
///
/// Then call `.auth()` to authenticate.
/// The call will setup `api_key`, `token`, `user_name`, `operator_id` for following `.get_data_entries()` calls.
///
/// # Example
///
/// ```no_run
/// use soracom_harvest_api_client::client::{Data, SoracomHarvestClient};
/// use soracom_harvest_api_client::endpoint::Endpoint;
///
/// let auth_key_id = "keyId-xxxxx";
/// let auth_key_secret = "secret-xxxxx";
///
/// let client: SoracomHarvestClient = SoracomHarvestClient::builder()
///    .auth_key_id(auth_key_id)
///    .auth_key_secret(auth_key_secret)
///    .endpoint(Endpoint::Japan)
///    .build();
/// let client = client.auth().unwrap();
///
/// let data: Vec<Data> = client
///    .get_data_entries("44010xxxxxxxxx", Some(1669023364195), Some(1669023464195), Some(50))
///    .unwrap();
/// ```

#[derive(TypedBuilder)]
pub struct SoracomHarvestClient {
    #[builder(setter(into))]
    auth_key_id: String,
    #[builder(setter(into))]
    auth_key_secret: String,
    /// Endpoint for this client.
    #[builder(setter(into), default = Endpoint::Global)]
    pub endpoint: Endpoint,
    #[builder(default)]
    api_key: String,
    #[builder(default)]
    token: String,
    /// User name for the authentication information.
    #[builder(default)]
    pub user_name: Option<String>,
    #[builder(default)]
    /// Operator ID for the authentication information.
    pub operator_id: Option<String>,
    #[builder(default)]
    client: Client,
}

impl Display for SoracomHarvestClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "- Endpoint: {}\n- API key: {}...\n- Token: {}...\n- User name: {:?}\n- Operator ID: {:?}",
            self.endpoint,
            self.api_key
                .chars()
                .into_iter()
                .take(20)
                .collect::<String>(),
            self.token.chars().into_iter().take(20).collect::<String>(),
            self.user_name,
            self.operator_id
        )
    }
}

impl SoracomHarvestClient {
    /// Authenticate with `auth_key_id` and `auth_key_secret` which were provided while creating a struct with `.builder()`.
    pub fn auth(&self) -> Result<Self, SoracomHarvestClientError> {
        let response = self
            .client
            .post(format!("{}/v1/auth", self.endpoint))
            .json(&AuthRequest {
                auth_key_id: self.auth_key_id.clone(),
                auth_key: self.auth_key_secret.clone(),
            })
            .send()?
            .json::<AuthResponse>()?;

        Ok(SoracomHarvestClient {
            auth_key_id: self.auth_key_id.clone(),
            auth_key_secret: self.auth_key_secret.clone(),
            endpoint: self.endpoint.clone(),
            api_key: response.api_key,
            token: response.token,
            user_name: response.user_name,
            operator_id: response.operator_id,
            client: self.client.clone(),
        })
    }

    /// Returns a vec of data entries sent from a SIM based on IMSI provided.
    /// Sort order is always descending (latest data entry first). No pagination support.
    ///
    /// - `imsi`: IMSI of the target SIM.
    /// - `from`: Start time for the data entries search range (unix time in milliseconds).
    /// - `to`: End time for the data entries search range (unix time in milliseconds).
    /// - `limit`: Maximum number of data entries to retrieve. Should be between 1 and 1000.
    pub fn get_data_entries(
        &self,
        imsi: impl Into<String>,
        from: Option<i64>,
        to: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<Data>, SoracomHarvestClientError> {
        let from = from.unwrap_or_else(|| (Utc::now() - Duration::days(1)).timestamp_millis());
        let to = to.unwrap_or_else(|| Utc::now().timestamp_millis());
        let limit = limit.unwrap_or(100);

        let response: HarvestDataResponse = self
            .client
            .get(format!(
                "{}/v1/data/Subscriber/{}",
                &self.endpoint,
                imsi.into()
            ))
            .header(USER_AGENT, "libshsqlite")
            .header("X-Soracom-Api-Key", &self.api_key)
            .header("X-Soracom-Token", &self.token)
            .header("X-Soracom-Lang", "en")
            .query(&[
                ("from", from.to_string()),
                ("to", to.to_string()),
                ("sort", "desc".to_string()),
                ("limit", limit.to_string()),
            ])
            .send()?
            .json()?;

        let mut result: Vec<Data> = Vec::new();
        for d in response.data {
            result.push(Data {
                content: Self::try_decode(d.content),
                content_type: d.content_type,
                time: d.time,
            })
        }

        Ok(result)
    }

    /// Deletes a data entry identified with IMSI and timestamp.
    ///
    /// - `imsi`: IMSI of the target SIM.
    /// - `time`: Timestamp of the target data entry to delete (unix time in milliseconds).
    pub fn delete_data_entry(
        &self,
        imsi: impl Into<String>,
        time: i64,
    ) -> Result<(), SoracomHarvestClientError> {
        self.client
            .delete(format!(
                "{}/v1/data/Subscriber/{}/{}",
                &self.endpoint,
                imsi.into(),
                time
            ))
            .header(USER_AGENT, "libshsqlite")
            .header("X-Soracom-Api-Key", &self.api_key)
            .header("X-Soracom-Token", &self.token)
            .header("X-Soracom-Lang", "en")
            .send()?;

        Ok(())
    }

    fn try_decode(content: String) -> String {
        // If value of the "content" property is like {"payload": "value"}, it could be base64-encoded data.
        if let Ok(base64_encoded_payload) =
            serde_json::from_str::<Base64EncodedPayload>(content.as_str())
        {
            // If value of the "payload" property can be decoded as base64
            if let Ok(decoded) = base64::decode(base64_encoded_payload.payload) {
                // and can be decoded as UTF-8 string,
                if let Ok(str) = String::from_utf8(decoded) {
                    // and the decoded string has only ASCII printable characters,
                    if str.chars().all(|c| matches!(c as u8, 0x20..=0x7E)) {
                        // return {"value": "<decoded string>"} as the content.
                        return format!(r#"{{"value":"{str}"}}"#);
                    }
                }
            }
        }
        // Otherwise return original content as is.
        content
    }
}

#[cfg(test)]
mod tests {
    use crate::client::SoracomHarvestClient;

    #[test]
    fn test_try_decode() {
        // valid base64
        assert_eq!(
            SoracomHarvestClient::try_decode(r#"{"payload":"aGVsbG8="}"#.to_string()),
            r#"{"value":"hello"}"#,
        );

        // invalid base64
        assert_eq!(
            SoracomHarvestClient::try_decode(r#"{"payload":"aGVsbG"}"#.to_string()),
            r#"{"payload":"aGVsbG"}"#,
        );

        // not ASCII printable ('\012\033')
        assert_eq!(
            SoracomHarvestClient::try_decode(r#"{"payload":"ChsK"}"#.to_string()),
            r#"{"payload":"ChsK"}"#,
        );

        // plain JSON
        assert_eq!(
            SoracomHarvestClient::try_decode(r#"{"temperature":20}"#.to_string()),
            r#"{"temperature":20}"#,
        );
    }
}
