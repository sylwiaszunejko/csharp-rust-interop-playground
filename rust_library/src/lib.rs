use future::CassResultValue;
use scylla::SessionBuilder;
use tokio::runtime::Runtime;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use crate::future::CassFuture;
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;

pub mod future;

pub static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

#[no_mangle]
pub extern "C" fn rust_hello_world(uri: *const c_char) -> *mut c_char {
    println!("Hello, World!");

    // Convert the raw C string to a Rust string
    let uri = unsafe {
        assert!(!uri.is_null());
        CStr::from_ptr(uri).to_string_lossy().into_owned()
    };

    // Run the async function synchronously
    let result = RUNTIME.block_on(async {
        match SessionBuilder::new().known_node(uri).build().await {
            Ok(_) => "Success".to_string(),
            Err(err) => {
                println!("{}", err);
                return "Error".to_string()
            }
        }
    });

    // Convert the Rust string back into a C string
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn cass_rust_hello_world() -> *const CassFuture {
    println!("Hello, World!");

    CassFuture::make_raw(async move {
        println!("Sleeping for 1 seconds...");
        thread::sleep(Duration::from_secs(1));
        println!("Done sleeping!");

        Ok(CassResultValue::Empty)
    })
}
