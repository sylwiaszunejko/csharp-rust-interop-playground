use scylla::SessionBuilder;
use tokio;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn rust_hello_world(uri: *const c_char) -> *mut c_char {
    println!("Hello, World!");

    // Convert the raw C string to a Rust string
    let uri = unsafe {
        assert!(!uri.is_null());
        CStr::from_ptr(uri).to_string_lossy().into_owned()
    };

    // Create a blocking runtime for executing async code in a synchronous function
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Run the async function synchronously
    let result = runtime.block_on(async {
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

