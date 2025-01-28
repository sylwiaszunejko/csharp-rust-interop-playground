use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use scylla::SessionBuilder;
use scylla::transport::errors::NewSessionError;
use scylla::transport::session::{CurrentDeserializationApi, GenericSession};
use crate::arfcffi::ArcFFI;
use crate::future::{CassFuture, CassFutureResult};

impl ArcFFI for GenericSession<CurrentDeserializationApi> {}
impl ArcFFI for CassFuture<GenericSession<CurrentDeserializationApi>, NewSessionError> {}

impl ArcFFI for CassFuture<(),()> {}

#[allow(unused)]
trait CheckSendSync: Send + Sync {}
impl CheckSendSync for CassFutureResult<GenericSession<CurrentDeserializationApi>, NewSessionError> {}

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
    }).into_ptr() as *const c_void
}

#[no_mangle]
pub unsafe extern "C" fn session_future_ready(ptr: *const c_void) -> bool {
    if ptr.is_null() {
        return false;
    }
    unsafe { &mut *(ptr as *mut CassFuture<GenericSession<CurrentDeserializationApi>, NewSessionError>) }.is_ready()
}

#[no_mangle]
pub unsafe extern "C" fn session_future_free(ptr: *const c_void) {
    if ptr.is_null() {
        return;
    }
    let fut = unsafe { &mut *(ptr as *mut CassFuture<GenericSession<CurrentDeserializationApi>, NewSessionError>) };
    ArcFFI::free(fut);
}
