#[cfg(test)]
use chrono::Utc;
use rusqlite::{Connection, LoadExtensionGuard};
use serde::Deserialize;
use soracom_harvest_api_client::client::{Data, SoracomHarvestClient};
use soracom_harvest_client::{send_http_message, send_udp_message};
use std::{env, error::Error, path::PathBuf, thread, time::Duration};

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
fn e2e() -> Result<(), Box<dyn Error>> {
    let config = match envy::prefixed("LIBSHSQLITE_").from_env::<Config>() {
        Ok(c) => c,
        Err(why) => panic!("{why}"),
    };

    let coverage = config.endpoint.unwrap_or_else(|| "global".to_string());
    let client: SoracomHarvestClient = SoracomHarvestClient::builder()
        .auth_key_id(config.auth_key_id)
        .auth_key_secret(config.auth_key_secret)
        .endpoint(coverage.as_str())
        .build();
    let client = client.auth()?;

    let (from, to) = send_test_data()?;

    let conn = Connection::open_in_memory()?;
    load_extension(&conn)?;

    conn.execute(
        format!(
            r#"CREATE VIRTUAL TABLE harvest_data USING shsqlite(IMSI '{}', FROM '{}', TO '{}', COVERAGE '{}');"#,
            config.imsi, from, to, coverage
        )
        .as_str(),
        (),
    )?;

    let mut harvest_data = Vec::new();
    let mut stmt = conn.prepare("SELECT * FROM harvest_data;")?;
    let result = stmt.query_map([], |row| {
        Ok(Data {
            time: row.get::<_, i64>(0)?,
            content_type: row.get(1)?,
            content: row.get(2)?,
        })
    })?;

    for d in result {
        harvest_data.push(d?);
    }

    assert_eq!(harvest_data.len(), 2);

    assert_eq!(harvest_data[0].content_type, "application/json");
    assert_eq!(harvest_data[0].content, r#"{"temperature":4096}"#);

    assert_eq!(harvest_data[1].content_type, "application/json");
    assert_eq!(
        harvest_data[1].content,
        r#"{"value":"hello from extension_test.rs"}"#
    );

    client
        .delete_data_entry(&config.imsi, harvest_data[0].time)
        .expect("Failed to delete test entry");
    client
        .delete_data_entry(&config.imsi, harvest_data[1].time)
        .expect("Failed to delete test entry");

    Ok(())
}

#[cfg(test)]
fn load_extension(conn: &Connection) -> rusqlite::Result<()> {
    let path_buf: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "..",
        "target",
        "debug",
        "libshsqlite",
    ]
    .iter()
    .collect();
    println!("{:?}", path_buf.as_path().as_os_str());

    unsafe {
        let _guard = LoadExtensionGuard::new(conn)?;
        conn.load_extension(path_buf.as_path().as_os_str(), None)
    }
}

#[cfg(test)]
fn send_test_data() -> Result<(i64, i64), Box<dyn Error>> {
    let interval = Duration::from_secs(1);

    let from = Utc::now().timestamp_millis();
    thread::sleep(interval);

    send_udp_message("hello from extension_test.rs")?;
    thread::sleep(interval);

    send_http_message(r#"{"temperature":4096}"#)?;
    thread::sleep(interval);

    let to = Utc::now().timestamp_millis();

    Ok((from, to))
}
