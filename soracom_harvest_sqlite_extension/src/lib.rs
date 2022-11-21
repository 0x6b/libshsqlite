//! A SQLite extension for Soracom Harvest Data.
//!
//! # Usage
//! ```shell
//! $ cargo build --release
//! $ # Setup required environment variables with the credential
//! $ export LIBSHSQLITE_AUTH_KEY_ID=keyId-xxxxx
//! $ export LIBSHSQLITE_AUTH_KEY_SECRET=secret-xxxxx
//! $ # Launch SQLite, load the extension, create a virtual table for your SIM
//! $ sqlite3
//! sqlite> .load target/release/libshsqlite
//! sqlite> CREATE VIRTUAL TABLE harvest_data USING shsqlite(IMSI '44120xxxxxxxxxx', COVERAGE `japan`);
//! sqlite> SELECT * FROM harvest_data;
//! time           content_type      value
//! -------------  ----------------  ----------------------------------------
//! 1669024327201  application/json  {"temperature":4096}
//! 1669024325202  application/json  {"value":"hello from extension_test.rs"}
//! sqlite> SELECT WHERE value->>'$.temperature' > 10;
//! time           content_type      value
//! -------------  ----------------  --------------------
//! 1669024327201  application/json  {"temperature":4096}
//! ```
//!
//! # SQLite3 virtual table arguments
//!
//! | Argument   | Description                                                               | Default             | Required |
//! |------------|---------------------------------------------------------------------------|---------------------|:--------:|
//! | `IMSI`     | Your IMSI                                                                 | None                |    x     |
//! | `FROM`     | Start time for the data entries search range (unix time in milliseconds). | 1 days ago from now |          |
//! | `TO`       | End time for the data entries search range (unix time in milliseconds).   | now                 |          |
//! | `COVERAGE` | Your SIM's coverage (`global` or `japan`)                                 | `global`            |          |
//! | `LIMIT`    | Maximum number of data entries to retrieve. Should be between 1 and 1000. | 100                 |          |
//!
//! ## Example
//!
//! ```sql
//! CREATE VIRTUAL TABLE harvest_data USING shsqlite(
//!     IMSI '...',
//!     FROM '...',
//!     TO '...',
//!     COVERAGE 'japan',
//!     LIMIT '...',
//! );
//! ```

pub mod error;
mod harvest_data_client;
mod module; // SQLite extension entry point
mod module_arguments_parser;
mod sqlite3ext;
