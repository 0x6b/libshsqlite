//! Deadly simple client library for Soracom Harvest Data. Provides simple functions to send a message with following protocols:
//! - HTTP
//! - UDP

use reqwest::{
    blocking::Client,
    header::{CONTENT_TYPE, USER_AGENT},
};
use std::{error::Error, net::UdpSocket, time::Duration};

const SORACOM_HARVEST_HTTP_ENDPOINT: &str = "http://harvest.soracom.io";
const SORACOM_HARVEST_TCP_UDP_ENDPOINT: &str = "harvest.soracom.io:8514";

/// Send a message to Soracom Harvest Data via HTTP. Roughly equivalents to:
///
/// ```shell
/// curl -X POST \
///      -H "user-agent:soracom_harvest_client" \
///      -H "content-type:application/json" \
///      -d "body" \
///      http://harvest.soracom.io
/// ```
pub fn send_http_message(body: impl Into<String>) -> Result<(), Box<dyn Error>> {
    Client::new()
        .post(SORACOM_HARVEST_HTTP_ENDPOINT)
        .header(USER_AGENT, "soracom_harvest_api_client")
        .header(CONTENT_TYPE, "application/json")
        .body(body.into())
        .send()?;

    Ok(())
}

/// Send a message to Soracom Harvest Data via UDP. Equivalents to:
/// ```shell
/// echo -n "data" | nc -u -w5 harvest.soracom.io 8514
/// ```
pub fn send_udp_message(data: impl Into<String>) -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_write_timeout(Some(Duration::from_secs(5)))?;
    socket.send_to(data.into().as_bytes(), SORACOM_HARVEST_TCP_UDP_ENDPOINT)?;

    Ok(())
}
