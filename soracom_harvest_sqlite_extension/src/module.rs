//! SQLite extension entry point.

use crate::{
    error::error_to_sqlite3_string,
    harvest_data_client::{HarvestDataClient, HarvestDataReader},
    module_arguments_parser::collect_options_from_args,
    sqlite3ext::{
        sqlite3, sqlite3_api_routines, sqlite3_context, sqlite3_index_info, sqlite3_int64,
        sqlite3_module, sqlite3_value, sqlite3_vtab, sqlite3_vtab_cursor, SQLITE_ERROR, SQLITE_OK,
        SQLITE_OK_LOAD_PERMANENTLY,
    },
};
use serde::Deserialize;
use soracom_harvest_api_client::client::SoracomHarvestClient;
use std::{
    ffi::{c_char, c_int, c_longlong, c_void, CString},
    sync::{Arc, Mutex},
};

#[derive(Deserialize, Debug)]
struct Config {
    auth_key_id: String,
    auth_key_secret: String,
}

#[no_mangle]
static mut SQLITE3_API: *mut sqlite3_api_routines = std::ptr::null_mut();

#[repr(C)]
struct Module {
    base: sqlite3_module,
    name: &'static [u8],
}

const SHSQLITE_MODULE: Module = Module {
    base: sqlite3_module {
        iVersion: 0,
        xCreate: Some(shsqlite_create),
        xConnect: Some(shsqlite_connect),
        xBestIndex: Some(shsqlite_best_index),
        xDisconnect: Some(shsqlite_disconnect),
        xDestroy: Some(shsqlite_destroy),
        xOpen: Some(shsqlite_open),
        xClose: Some(shsqlite_close),
        xFilter: Some(shsqlite_filter),
        xNext: Some(shsqlite_next),
        xEof: Some(shsqlite_eof),
        xColumn: Some(shsqlite_column),
        xRowid: Some(shsqlite_rowid),
        xUpdate: None,
        xBegin: None,
        xSync: None,
        xCommit: None,
        xRollback: None,
        xFindFunction: None,
        xRename: None,
        xSavepoint: None,
        xRelease: None,
        xRollbackTo: None,
        xShadowName: None,
    },
    name: b"shsqlite\0",
};

#[repr(C)]
struct VirtualTable {
    pub base: sqlite3_vtab,
    pub data: Arc<Mutex<HarvestDataClient>>,
}

#[repr(C)]
struct VirtualCursor {
    pub base: sqlite3_vtab_cursor,
    pub reader: Arc<Mutex<HarvestDataReader>>,
}

#[no_mangle]
unsafe extern "C" fn register_module(
    db: *mut sqlite3,
    pz_err_msg: *mut *mut c_char,
    p_api: *mut sqlite3_api_routines,
) -> c_int {
    let result = ((*p_api).create_module.unwrap())(
        db,
        SHSQLITE_MODULE.name.as_ptr() as *const c_char,
        &SHSQLITE_MODULE as *const Module as *const sqlite3_module,
        std::ptr::null_mut(),
    );

    match result {
        SQLITE_OK => SQLITE_OK_LOAD_PERMANENTLY,
        _ => {
            let err = format!("Failed to create module, status: {}", result);
            if let Some(ptr) = error_to_sqlite3_string(SQLITE3_API, err) {
                *pz_err_msg = ptr;
            }
            SQLITE_ERROR
        }
    }
}

#[no_mangle]
unsafe extern "C" fn sqlite3_shsqlite_init(
    db: *mut sqlite3,
    pz_err_msg: *mut *mut c_char,
    p_api: *mut sqlite3_api_routines,
) -> c_int {
    SQLITE3_API = p_api;

    let result = register_module(db, pz_err_msg, p_api);
    match result {
        SQLITE_OK => {
            let result = ((*p_api).auto_extension.unwrap())(Some(std::mem::transmute(
                register_module as *const (),
            )));
            if result != SQLITE_OK {
                return result;
            }
        }
        _ => return result,
    }

    SQLITE_OK_LOAD_PERMANENTLY
}

#[no_mangle]
unsafe extern "C" fn shsqlite_create(
    db: *mut sqlite3,
    _p_aux: *mut c_void,
    argc: c_int,
    argv: *const *const c_char,
    pp_vtab: *mut *mut sqlite3_vtab,
    pz_err: *mut *mut c_char,
) -> c_int {
    let config = match envy::prefixed("LIBSHSQLITE_").from_env::<Config>() {
        Ok(c) => c,
        Err(why) => panic!("{why}"),
    };

    match collect_options_from_args(argc, argv) {
        Ok((imsi, endpoint, from, to, limit)) => {
            let client = SoracomHarvestClient::builder()
                .auth_key_id(config.auth_key_id)
                .auth_key_secret(config.auth_key_secret)
                .endpoint(endpoint)
                .build();

            let mut harvest_data = HarvestDataClient::builder()
                .client(client)
                .imsi(imsi)
                .from(from)
                .to(to)
                .limit(limit)
                .build();

            match harvest_data.open() {
                Ok(_) => {
                    let result = declare_table(
                        db,
                        SQLITE3_API,
                        vec![
                            "time INTEGER".to_string(),
                            "content_type TEXT".to_string(),
                            "value TEXT".to_string(),
                        ],
                    );
                    let p_new = Box::new(VirtualTable {
                        base: sqlite3_vtab {
                            pModule: std::ptr::null_mut(),
                            nRef: 0,
                            zErrMsg: std::ptr::null_mut(),
                        },
                        data: Arc::new(Mutex::new(harvest_data)),
                    });
                    *pp_vtab = Box::into_raw(p_new) as *mut sqlite3_vtab;
                    result
                }
                Err(err) => {
                    if let Some(ptr) = error_to_sqlite3_string(SQLITE3_API, err) {
                        *pz_err = ptr;
                    }
                    SQLITE_ERROR
                }
            }
        }
        Err(_) => SQLITE_ERROR,
    }
}

#[no_mangle]
unsafe extern "C" fn shsqlite_connect(
    db: *mut sqlite3,
    p_aux: *mut c_void,
    argc: c_int,
    argv: *const *const c_char,
    pp_vtab: *mut *mut sqlite3_vtab,
    pz_err: *mut *mut c_char,
) -> c_int {
    shsqlite_create(db, p_aux, argc, argv, pp_vtab, pz_err)
}

#[no_mangle]
unsafe extern "C" fn shsqlite_best_index(
    _p_vtab: *mut sqlite3_vtab,
    _arg1: *mut sqlite3_index_info,
) -> c_int {
    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn shsqlite_disconnect(p_vtab: *mut sqlite3_vtab) -> c_int {
    shsqlite_destroy(p_vtab)
}

#[no_mangle]
unsafe extern "C" fn shsqlite_destroy(p_vtab: *mut sqlite3_vtab) -> c_int {
    if !p_vtab.is_null() {
        let table = Box::from_raw(p_vtab as *mut VirtualTable);
        drop(table);
    }

    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn shsqlite_open(
    p_vtab: *mut sqlite3_vtab,
    pp_cursor: *mut *mut sqlite3_vtab_cursor,
) -> c_int {
    let table = &mut *(p_vtab as *mut VirtualTable);
    let data = Arc::clone(&table.data);
    let mut lock = data.lock().unwrap();
    let reader = lock.get_reader();

    let cursor = Box::new(VirtualCursor {
        base: sqlite3_vtab_cursor { pVtab: p_vtab },
        reader: Arc::new(Mutex::new(reader)),
    });
    *pp_cursor = Box::into_raw(cursor) as _;

    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn shsqlite_close(p_cursor: *mut sqlite3_vtab_cursor) -> c_int {
    if !p_cursor.is_null() {
        let cursor = Box::from_raw(p_cursor as *mut VirtualCursor);
        drop(cursor);
    }

    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn shsqlite_filter(
    _arg1: *mut sqlite3_vtab_cursor,
    _idx_num: c_int,
    _idx_str: *const c_char,
    _argc: c_int,
    _argv: *mut *mut sqlite3_value,
) -> c_int {
    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn shsqlite_next(p_cursor: *mut sqlite3_vtab_cursor) -> c_int {
    let cursor = &mut *(p_cursor as *mut VirtualCursor);
    let lock = Arc::clone(&cursor.reader);
    let mut reader = lock.lock().unwrap();

    reader.move_next();

    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn shsqlite_eof(p_cursor: *mut sqlite3_vtab_cursor) -> c_int {
    let cursor = &mut *(p_cursor as *mut VirtualCursor);
    let lock = Arc::clone(&cursor.reader);
    let reader = lock.lock().unwrap();

    if reader.has_value() {
        0
    } else {
        1
    }
}

#[no_mangle]
unsafe extern "C" fn shsqlite_column(
    p_cursor: *mut sqlite3_vtab_cursor,
    p_context: *mut sqlite3_context,
    column: c_int,
) -> c_int {
    let cursor = &mut *(p_cursor as *mut VirtualCursor);
    let lock = Arc::clone(&cursor.reader);
    let reader = lock.lock().unwrap();

    yield_cell_value(p_context, SQLITE3_API, reader.get_value(column as usize));

    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn shsqlite_rowid(
    p_cursor: *mut sqlite3_vtab_cursor,
    p_rowid: *mut sqlite3_int64,
) -> c_int {
    let cursor = &mut *(p_cursor as *mut VirtualCursor);
    let lock = Arc::clone(&cursor.reader);
    let reader = lock.lock().unwrap();

    *p_rowid = reader.get_index() as c_longlong;

    SQLITE_OK
}

unsafe fn declare_table(
    db: *mut sqlite3,
    api: *mut sqlite3_api_routines,
    columns: Vec<String>,
) -> c_int {
    ((*api).declare_vtab.unwrap())(db, create_declare_table_statement(columns).as_ptr() as _)
}

fn create_declare_table_statement(columns: Vec<String>) -> CString {
    CString::new(format!(
        "CREATE TABLE harvest_data ({})",
        columns.join(", ")
    ))
    .unwrap()
}

unsafe fn yield_cell_value(
    p_context: *mut sqlite3_context,
    api: *mut sqlite3_api_routines,
    value: String,
) {
    match value.parse::<i64>() {
        Ok(i) => ((*api).result_int64.unwrap())(p_context, i),
        Err(_) => {
            let (len, raw) = to_raw_string(value);
            ((*api).result_text.unwrap())(p_context, raw, len as c_int, Some(destructor))
        }
    }
}

fn to_raw_string(s: String) -> (usize, *mut c_char) {
    let cstr = CString::new(s.as_str().as_bytes()).unwrap();
    let len = cstr.as_bytes().len();
    let raw = cstr.into_raw();

    (len, raw)
}

unsafe extern "C" fn destructor(raw: *mut c_void) {
    drop(CString::from_raw(raw as *mut c_char));
}
