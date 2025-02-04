use std::error::Error;
use crate::arfcffi::ArcFFI;
use crate::ffi;
use crate::future::{CassFuture, CassFutureResult};
use crate::result::WQueryResult;
use scylla::transport::errors::{NewSessionError, QueryError};
use scylla::transport::query_result::IntoRowsResultError;
use scylla::transport::session::{CurrentDeserializationApi, GenericSession};
use scylla::{QueryResult, QueryRowsResult, SessionBuilder};
use std::ffi::{c_void, CStr, CString};
use std::fmt::Error;
use std::os::raw::c_char;
use std::ptr::null;
use std::time;
use futures::StreamExt;

impl ArcFFI for GenericSession<CurrentDeserializationApi> {}
impl ArcFFI for CassFuture<GenericSession<CurrentDeserializationApi>, NewSessionError> {}
impl ArcFFI for CassFuture<QueryRowsResult, IntoRowsResultError> {}
impl ArcFFI for CassFuture<QueryResult, QueryError> {}

impl ArcFFI for CassFuture<(), ()> {}
impl ArcFFI for CassFutureResult<(), ()> {}

#[allow(unused)]
trait CheckSendSync: Send + Sync {}
impl CheckSendSync
    for CassFutureResult<GenericSession<CurrentDeserializationApi>, NewSessionError>
{
}
impl CheckSendSync for CassFutureResult<QueryRowsResult, IntoRowsResultError> {}

#[no_mangle]
pub extern "C" fn create_session(uri: *const c_char, id: *const c_char) -> *const c_void {
    // Convert the raw C string to a Rust string
    let uri = unsafe {
        assert!(!uri.is_null());
        CStr::from_ptr(uri).to_string_lossy().into_owned()
    };

    let id = unsafe {
        assert!(!id.is_null());
        CStr::from_ptr(id).to_string_lossy().into_owned()
    };

    println!("Hello, World! {}", id);

    CassFuture::new_from_future(async move {
        println!("Create Session... {}", id);
        let session = SessionBuilder::new().known_node(uri).build().await;
        println!("Session created! {}", id);
        session
    })
    .into_ptr() as *const c_void
}

#[no_mangle]
pub unsafe extern "C" fn session_future_ready(ptr: *const c_void) -> bool {
    if ptr.is_null() {
        return false;
    }
    unsafe {
        &mut *(ptr as *mut CassFuture<GenericSession<CurrentDeserializationApi>, NewSessionError>)
    }
    .is_ready()
}

#[no_mangle]
pub unsafe extern "C" fn session_future_get_result(ptr: *const c_void) -> *const c_void {
    if ptr.is_null() {
        return std::ptr::null();
    }
    let fut = unsafe {
        &mut *(ptr as *mut CassFuture<GenericSession<CurrentDeserializationApi>, NewSessionError>)
    };
    match *fut.result.lock().unwrap() {
        CassFutureResult::Result(ref res) => res as *const _ as *const c_void,
        CassFutureResult::Error(ref err) => {
            let error_message = CString::new(err.to_string()).unwrap();
            error_message.into_raw() as *const c_void
        }
        _ => std::ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn session_future_get_error(ptr: *const c_void) -> *const c_char {
    if ptr.is_null() {
        return null();
    }
    let fut = unsafe {
        &mut *(ptr as *mut CassFuture<GenericSession<CurrentDeserializationApi>, NewSessionError>)
    };
    match *fut.result.lock().unwrap() {
        CassFutureResult::Error(ref err) => ffi::string_to_c_char(err.to_string().as_str()),
        _ => null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn query_future_ready(ptr: *const c_void) -> bool {
    if ptr.is_null() {
        return false;
    }
    unsafe { &mut *(ptr as *mut CassFuture<QueryResult, QueryError>) }.is_ready()
}

#[no_mangle]
pub unsafe extern "C" fn query_future_get_result(ptr: *const c_void) -> *const c_void {
    if ptr.is_null() {
        return std::ptr::null();
    }
    let fut = unsafe { &mut *(ptr as *mut CassFuture<QueryResult, QueryError>) };
    match *fut.result.lock().unwrap() {
        CassFutureResult::Result(ref res) => res as *const _ as *const c_void,
        CassFutureResult::Error(ref err) => {
            let error_message = CString::new(err.to_string()).unwrap();
            error_message.into_raw() as *const c_void
        }
        _ => std::ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn execute_query(ptr: *const c_void, query: *const c_char) -> *const c_void {
    if ptr.is_null() {
        return std::ptr::null();
    }

    // Convert the raw C string to a Rust string
    let query = unsafe {
        assert!(!query.is_null());
        CStr::from_ptr(query).to_string_lossy().into_owned()
    };

    let session = unsafe { &mut *(ptr as *mut GenericSession<CurrentDeserializationApi>) };
    CassFuture::new_from_future(async move || -> Result<WQueryResult, QueryError> {
        println!("Executing query... {}", query);
        let mut query_result = session
            .query_iter(query, ())
            .await.expect("")
            .rows_stream::<WQueryResult>().expect("");
        // let iter = session.query_iter(query, ()).await?.rows_stream::<(WQueryResult,)>();
        println!("Query executed!");
        query_result.next().await.expect("")
    })
    .into_ptr() as *const c_void
}

#[no_mangle]
pub unsafe extern "C" fn session_future_free(ptr: *const c_void) {
    if ptr.is_null() {
        return;
    }
    let fut = unsafe {
        &mut *(ptr as *mut CassFuture<GenericSession<CurrentDeserializationApi>, NewSessionError>)
    };
    ArcFFI::free(fut);
}

#[test]
fn test_1() {
    let session_future = create_session(
        ffi::string_to_c_char("172.17.0.2:9042"),
        ffi::string_to_c_char("1"),
    );
    unsafe {
        while session_future_ready(session_future) == false {
            std::thread::sleep(time::Duration::from_millis(100));
        }

        let err = session_future_get_error(session_future);
        assert!(
            err.is_null(),
            "session creation failed with {}",
            ffi::c_char_to_string(err)
        );

        let session = session_future_get_result(session_future);
        assert!(!session.is_null());

        let query_future = execute_query(
            session,
            ffi::string_to_c_char("select * from system.local;"),
        );

        while query_future_ready(query_future) == false {
            std::thread::sleep(time::Duration::from_millis(100));
        }

        let rows = query_future_get_result(query_future);

        assert!(rows.is_null());
    }
}
