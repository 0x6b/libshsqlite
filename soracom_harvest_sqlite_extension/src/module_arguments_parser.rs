//! SQLite option parser

use crate::error::{
    ArgumentError,
    ArgumentError::{InvalidFrom, InvalidLimit, InvalidTo, NoImsi, UnknownOption},
};
use chrono::{Duration, Utc};
use regex::Regex;
use soracom_harvest_api_client::endpoint::Endpoint;
use std::ffi::{c_char, c_int, CStr};

enum ModuleArgument {
    Imsi(String),       // required
    Coverage(Endpoint), // optional
    From(i64),          // optional
    To(i64),            // optional
    Limit(u32),         // optional, and should be between 1 to 1000
}

pub(crate) unsafe fn collect_options_from_args(
    argc: c_int,
    argv: *const *const c_char,
) -> Result<(String, Endpoint, i64, i64, u32), ArgumentError> {
    let mut imsi = "".to_string();
    let mut endpoint = Endpoint::default();
    let mut from = 0i64;
    let mut to = 0i64;
    let mut limit = 100u32;

    for arg in collect_strings_from_raw(argc as usize, argv) {
        if let Ok(option) = parse_option(arg.as_str()) {
            match option {
                ModuleArgument::Imsi(s) => imsi = s.to_string(),
                ModuleArgument::Coverage(e) => endpoint = e,
                ModuleArgument::From(i) => from = i,
                ModuleArgument::To(i) => to = i,
                ModuleArgument::Limit(u) => limit = u,
            }
        }
    }

    if imsi.is_empty() {
        return Err(NoImsi);
    }

    if from == 0 {
        from = (Utc::now() - Duration::days(1)).timestamp_millis();
    }

    if to == 0 {
        to = Utc::now().timestamp_millis();
    }

    if limit < 1 && limit > 1000 {
        return Err(InvalidLimit);
    }

    Ok((imsi, endpoint, from, to, limit))
}

unsafe fn collect_strings_from_raw(n: usize, args: *const *const c_char) -> Vec<String> {
    let mut vec = Vec::with_capacity(n);

    let args = args as *mut *const c_char;
    for i in 0..n {
        let arg = *(args.add(i));
        let s = read_string_from_raw(arg);
        vec.push(s);
    }

    vec
}

unsafe fn read_string_from_raw(raw: *const c_char) -> String {
    let cstr = CStr::from_ptr(raw);
    cstr.to_str().unwrap_or_default().to_string()
}

fn parse_option(input: &str) -> Result<ModuleArgument, ArgumentError> {
    if let Ok(re) = Regex::new(r#"(?i)^(IMSI|COVERAGE|FROM|TO|LIMIT)\s+['"]([^'"]+)['"]$"#) {
        if let Some(cap) = re.captures(input) {
            return match cap[1].to_lowercase().as_str() {
                "imsi" => Ok(ModuleArgument::Imsi(cap[2].into())),
                "coverage" => Ok(ModuleArgument::Coverage(cap[2].into())),
                "from" => match cap[2].parse::<i64>() {
                    Ok(i) => Ok(ModuleArgument::From(i)),
                    Err(_) => Err(InvalidFrom),
                },
                "to" => match cap[2].parse::<i64>() {
                    Ok(i) => Ok(ModuleArgument::To(i)),
                    Err(_) => Err(InvalidTo),
                },
                "limit" => match cap[2].parse::<u32>() {
                    Ok(u) => Ok(ModuleArgument::Limit(u)),
                    Err(_) => Err(InvalidLimit),
                },
                _ => Err(UnknownOption),
            };
        }
    }

    Err(UnknownOption)
}

#[cfg(test)]
mod tests {
    use crate::module_arguments_parser::collect_options_from_args;
    use soracom_harvest_api_client::endpoint::Endpoint;
    use std::{error::Error, ffi::CStr};

    #[test]
    fn test_collect_options_from_args() -> Result<(), Box<dyn Error>> {
        let out = vec![
            CStr::from_bytes_with_nul(b"IMSI '441200000050000'\0").unwrap(),
            CStr::from_bytes_with_nul(b"COVERAGE 'japan'\0").unwrap(),
            CStr::from_bytes_with_nul(b"FROM '1668003111681'\0").unwrap(),
            CStr::from_bytes_with_nul(b"TO '1668604289406'\0").unwrap(),
            CStr::from_bytes_with_nul(b"LIMIT '1000'\0").unwrap(),
        ]
        .into_iter()
        .map(|s| s.as_ptr())
        .collect::<Vec<_>>();

        unsafe {
            assert_eq!(
                (
                    "441200000050000".to_string(),
                    Endpoint::Japan,
                    1668003111681,
                    1668604289406,
                    1000
                ),
                collect_options_from_args(5, out.as_ptr())?
            )
        }

        Ok(())
    }

    #[test]
    fn test_collect_options_from_args_with_optional() {
        let out = vec![
            CStr::from_bytes_with_nul(b"IMSI '441200000050000'\0").unwrap(),
            CStr::from_bytes_with_nul(b"FROM '1668003111681'\0").unwrap(),
            CStr::from_bytes_with_nul(b"TO '1668604289406'\0").unwrap(),
        ]
        .into_iter()
        .map(|s| s.as_ptr())
        .collect::<Vec<_>>();

        unsafe {
            assert_eq!(
                (
                    "441200000050000".to_string(),
                    Endpoint::Global,
                    1668003111681,
                    1668604289406,
                    100
                ),
                collect_options_from_args(3, out.as_ptr()).unwrap()
            )
        }
    }
}
