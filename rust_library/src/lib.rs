use future::CassResultValue;
use scylla::SessionBuilder;
use scylla::Session;
use tokio::runtime::Runtime;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use crate::future::CassFuture;
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;
use futures::TryStreamExt;

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

#[no_mangle]
pub extern "C" fn async_connect_and_run_query(uri: *const c_char) -> *const CassFuture {
    println!("Hello, World!");

    // Convert the raw C string to a Rust string
    let uri = unsafe {
        assert!(!uri.is_null());
        CStr::from_ptr(uri).to_string_lossy().into_owned()
    };

    CassFuture::make_raw(async move {
        println!("Create Session...");

        let session: Session = SessionBuilder::new().known_node(uri).build().await.map_err(|err| (err.to_string()))?;

        println!("Connected to ScyllaDB!");

        println!("Create Keyspace");

        // Create a keyspace and table (if not already created)
        session
            .query_unpaged(
                "CREATE KEYSPACE IF NOT EXISTS ks WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1}",
                &[],
            )
            .await.map_err(|err| (err.to_string()))?;

        println!("Create Table");

        session
            .query_unpaged(
                "CREATE TABLE IF NOT EXISTS ks.extab (a int primary key)",
                &[],
            )
            .await.map_err(|err| (err.to_string()))?;

        println!("Insert Value");
        // Insert a value into the table
        let to_insert: i32 = 12345;
        session
            .query_unpaged("INSERT INTO ks.extab (a) VALUES(?)", (to_insert,))
            .await.map_err(|err| (err.to_string()))?;

        println!("Read Value");
        // Query rows from the table and print them
        let mut iter = session.query_iter("SELECT a FROM ks.extab", &[])
            .await.map_err(|err| (err.to_string()))?
            .rows_stream::<(i32,)>().map_err(|err| (err.to_string()))?;
        while let Some(read_row) = iter.try_next().await.map_err(|err| (err.to_string()))? {
            println!("Read a value from row: {}", read_row.0);
        }

        Ok(CassResultValue::Empty)
    })
}
