#![feature(box_as_ptr)]

use scylla::SessionBuilder;
use tokio::runtime::Runtime;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use crate::future::CassFuture;
use std::sync::{LazyLock};
use std::thread;
use std::time::Duration;
use std::ffi::c_void;
use crate::arfcffi::ArcFFI;

pub mod future;
mod arfcffi;
mod testing;
mod result;
mod session;
mod ffi;

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
pub extern "C" fn cass_rust_hello_world() -> *const c_void {
    println!("Hello, World!");

    CassFuture::<(), ()>::new_from_future(async move {
        println!("Sleeping for 1 seconds...");
        thread::sleep(Duration::from_secs(1));
        println!("Done sleeping!");
    }).as_ptr() as *const c_void
}

// #[no_mangle]
// pub extern "C" fn async_connect_and_run_query(uri: *const c_char, id: *const c_char) -> *const c_void {
//     // Convert the raw C string to a Rust string
//     let uri = unsafe {
//         assert!(!uri.is_null());
//         CStr::from_ptr(uri).to_string_lossy().into_owned()
//     };
//
//     let id = unsafe {
//         assert!(!id.is_null());
//         CStr::from_ptr(id).to_string_lossy().into_owned()
//     };
//
//     println!("Hello, World! {}", id);
//
//     CassFuture::make_raw(async move {
//         println!("Create Session... {}", id);
//
//         let session: Session = SessionBuilder::new().known_node(uri).build().await.map_err(|err| (err.to_string()))?;
//
//         println!("Connected to ScyllaDB! {}", id);
//
//         println!("Create Keyspace {}", id);
//
//         // Create a keyspace and table (if not already created)
//         session
//             .query_unpaged(
//                 "CREATE KEYSPACE IF NOT EXISTS ks WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1}",
//                 &[],
//             )
//             .await?;
//
//         println!("Create Table {}", id);
//
//         session
//             .query_unpaged(
//                 "CREATE TABLE IF NOT EXISTS ks.extab (a int primary key)",
//                 &[],
//             )
//             .await?;
//
//         println!("Insert Value {}", id);
//         // Insert a value into the table
//         let to_insert: i32 = 12345;
//         session
//             .query_unpaged("INSERT INTO ks.extab (a) VALUES(?)", (to_insert,))
//             .await?;
//
//         println!("Read Value {}", id);
//         // Query rows from the table and print them
//         let mut iter = session.query_iter("SELECT a FROM ks.extab", &[])
//             .await?
//             .rows_stream::<(i32,)>()?;
//         while let Some(read_row) = iter.try_next().await? {
//             println!("Read a value from row: {}, {}", read_row.0, id);
//         }
//     }).cast()
// }

// #[no_mangle]
// pub extern "C" fn async_run_query(future: *const CassFuture, query: *const c_char)
// {
//     // Convert the raw C string to a Rust string
//     let query = unsafe {
//         assert!(!query.is_null());
//         CStr::from_ptr(query).to_string_lossy().into_owned()
//     };
//
//     CassFuture::make_raw(async move {
//         println!("Run Query... {}", query);
//
//         session
//             .query_unpaged(
//                 "CREATE TABLE IF NOT EXISTS ks.extab (a int primary key)",
//                 &[],
//             )
//             .await.map_err(|err| (err.to_string()))?;
//
//         Ok(CassResultValue::Empty)
//     })
// }
