//! Error definition.

use crate::sqlite3ext::sqlite3_api_routines;
use std::{
    ffi::{c_char, c_int, CString},
    ptr::copy_nonoverlapping,
};
use thiserror::Error;

/// Possible errors while parsing arguments
#[derive(Debug, Error)]
pub enum ArgumentError {
    /// No IMSI is provided.
    #[error("No IMSI is provided")]
    NoImsi,

    /// Invalid `from` is provided.
    #[error("Invalid 'from' is provided")]
    InvalidFrom,

    /// Invalid `to` is provided.
    #[error("Invalid 'to' is provided")]
    InvalidTo,

    /// Invalid `limit` is provided. It should be from 1 to 1000.
    #[error("Invalid 'limit' is provided. It should be from 1 to 1000")]
    InvalidLimit,

    /// Unknown option is provided.
    #[error("Unknown option is provided")]
    UnknownOption,
}

/// Convert error message to SQLite3 string.
///
/// # Safety
/// Behavior is undefined if any of conditions from `copy_nonoverlapping` are violated.
pub(crate) unsafe fn error_to_sqlite3_string(
    api: *mut sqlite3_api_routines,
    err: impl Into<String>,
) -> Option<*mut c_char> {
    let cstr = CString::new(err.into()).ok()?;
    let len = cstr.as_bytes_with_nul().len();

    let ptr = ((*api).malloc.unwrap())(len as c_int) as *mut c_char;
    if !ptr.is_null() {
        copy_nonoverlapping(cstr.as_ptr(), ptr, len);
        Some(ptr)
    } else {
        None
    }
}
