//! Simple command-line client for Soracom Harvest Data. You have to use this from Soracom-connected device.
//!
//! # Usage
//!
//! soracom_harvest_client [FLAGS] [message]
//!
//! # Flags
//!
//! -h, --help       Prints help information
//!     --http       Use HTTP to send your message
//!     --udp        use UDP to send your message
//! -V, --version    Prints version information
//!
//! # Argument
//!
//! <message>    Message to sent. If none, sent CPUs temperature instead.

use soracom_harvest_client::{send_http_message, send_udp_message};
use std::{collections::HashMap, error::Error};
use structopt::StructOpt;
use sysinfo::{CpuExt, System, SystemExt};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "soracom_harvest_client",
    about = "Simple command-line client for Soracom Harvest Data. You have to use this from Soracom-connected device."
)]
struct Opt {
    #[structopt(long, group = "protocol")]
    /// Use HTTP to send your message.
    http: bool,

    #[structopt(long, group = "protocol")]
    /// use UDP to send your message.
    udp: bool,

    /// Message to sent. If none, sent CPUs temperature instead.
    #[structopt()]
    message: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt: Opt = Opt::from_args();
    let message = match opt.message {
        None => {
            let mut data = HashMap::new();
            for cpu in System::new_all().cpus() {
                data.insert(cpu.name().to_string(), cpu.cpu_usage());
            }
            serde_json::to_string(&data)?
        }
        Some(s) => s,
    };

    if opt.http {
        send_http_message(&message)?;
    } else if opt.udp {
        send_udp_message(&message)?;
    }

    println!("{} {}", chrono::Local::now().to_rfc3339(), message);
    Ok(())
}
