#[cfg(test)]
use chrono::Utc;
use serde::Deserialize;
use soracom_harvest_api_client::client::{Data, SoracomHarvestClient};
use soracom_harvest_client::{send_http_message, send_udp_message};
use std::{error::Error, thread, time::Duration};

#[derive(Deserialize, Debug)]
struct Config {
    auth_key_id: String,
    auth_key_secret: String,
    #[serde(rename = "test_imsi")]
    imsi: String,
    #[serde(rename = "test_endpoint")]
    endpoint: Option<String>,
}

#[test]
fn auth_and_get_data_entries() -> Result<(), Box<dyn Error>> {
    let config = match envy::prefixed("LIBSHSQLITE_").from_env::<Config>() {
        Ok(c) => c,
        Err(why) => panic!("{why}"),
    };

    let client: SoracomHarvestClient = SoracomHarvestClient::builder()
        .auth_key_id(config.auth_key_id)
        .auth_key_secret(config.auth_key_secret)
        .endpoint(config.endpoint.unwrap_or_default())
        .build();
    let client = client.auth()?;

    let (from, to) = send_test_data()?;

    let data: Vec<Data> = client.get_data_entries(&config.imsi, Some(from), Some(to), Some(50))?;

    assert_eq!(data.len(), 2);

    let mut iter = data.into_iter();

    let item = iter.next().ok_or("no item")?;
    let time = item.time;
    assert!(time > from);
    assert_eq!(item.content, r#"{"temperature":2048}"#);
    assert_eq!(item.content_type, "application/json");

    let item = iter.next().ok_or("no item")?;
    let time = item.time;
    assert!(time > from);
    assert_eq!(item.content, r#"{"value":"hello from client_test.rs"}"#);
    assert_eq!(item.content_type, "application/json");

    client
        .delete_data_entry(&config.imsi, time)
        .expect("Failed to delete test entry");
    client
        .delete_data_entry(&config.imsi, time)
        .expect("Failed to delete test entry");

    Ok(())
}

#[cfg(test)]
fn send_test_data() -> Result<(i64, i64), Box<dyn Error>> {
    let interval = Duration::from_secs(1);

    let from = Utc::now().timestamp_millis();
    thread::sleep(interval);

    send_udp_message("hello from client_test.rs".to_string())?;
    thread::sleep(interval);

    send_http_message(r#"{"temperature":2048}"#.to_string())?;
    thread::sleep(interval);

    let to = Utc::now().timestamp_millis();

    Ok((from, to))
}
