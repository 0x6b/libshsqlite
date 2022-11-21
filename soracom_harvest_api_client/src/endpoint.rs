//! SORACOM API endpoint declaration.
//!
//! # Example
//!
//! ```
//! use soracom_harvest_api_client::endpoint::Endpoint;
//!
//! let default = Endpoint::default();
//! let jp = Endpoint::Japan;
//! let g = Endpoint::from("global");
//!
//! assert_eq!(default.to_string(), "https://g.api.soracom.io");
//! assert_eq!(jp.to_string(), "https://api.soracom.io");
//! assert_eq!(g.to_string(), "https://g.api.soracom.io");
//! ```

use std::fmt::{Display, Formatter};

/// Endpoint representation, based on SORACOM coverage.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Endpoint {
    /// Global coverage
    Global,

    /// Japan coverage
    Japan,
}

impl Default for Endpoint {
    fn default() -> Self {
        Endpoint::Global
    }
}

impl Endpoint {
    /// Returns `&str` representation of the endpoint.
    pub fn as_str(&self) -> &str {
        match self {
            Endpoint::Global => "https://g.api.soracom.io",
            Endpoint::Japan => "https://api.soracom.io",
        }
    }
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for Endpoint {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "g" | "global" => Endpoint::Global,
            "jp" | "japan" => Endpoint::Japan,
            _ => Endpoint::Global,
        }
    }
}

impl From<String> for Endpoint {
    fn from(s: String) -> Self {
        Endpoint::from(s.as_str())
    }
}
