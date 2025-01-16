use std::sync::{Mutex, Condvar, Arc};
use std::os::raw::c_void;
use std::future::Future;
use crate::RUNTIME;
use tokio::task::JoinHandle;
use std::mem;
use std::os::raw::c_char;
use std::ffi::CString;

pub enum CassResultValue {
    Empty,
    QueryResult(Arc<String>),
    QueryError(Arc<String>),
}

type CassFutureError = String;

pub type CassFutureResult = Result<CassResultValue, CassFutureError>;

pub type CassFutureCallback =
    Option<unsafe extern "C" fn(future: *const CassFuture, data: *mut c_void)>;

// *mut c_void is not Send, so Rust will have to take our word
// that we won't screw something up
unsafe impl Send for BoundCallback {}

struct BoundCallback {
    pub cb: CassFutureCallback,
    pub data: *mut c_void,
}

impl BoundCallback {
    fn invoke(self, fut: &CassFuture) {
        unsafe {
            self.cb.unwrap()(fut as *const CassFuture, self.data);
        }
    }
}

#[derive(Default)]
struct CassFutureState {
    value: Option<CassFutureResult>,
    err_string: Option<String>,
    callback: Option<BoundCallback>,
    join_handle: Option<JoinHandle<()>>,
}

pub struct CassFuture {
    state: Mutex<CassFutureState>,
    wait_for_value: Condvar,
}

/// Defines a pointer manipulation API for shared heap-allocated data.
///
/// Implement this trait for types that require a shared ownership of data.
/// The data should be allocated via [`Arc::new`], and then returned to the user as a pointer.
/// The user is responsible for freeing the memory associated
/// with the pointer using corresponding driver's API function.
pub trait ArcFFI {
    fn as_ptr(self: &Arc<Self>) -> *const Self {
        #[allow(clippy::disallowed_methods)]
        Arc::as_ptr(self)
    }
    fn into_ptr(self: Arc<Self>) -> *const Self {
        #[allow(clippy::disallowed_methods)]
        Arc::into_raw(self)
    }
    unsafe fn from_ptr(ptr: *const Self) -> Arc<Self> {
        #[allow(clippy::disallowed_methods)]
        Arc::from_raw(ptr)
    }
    unsafe fn cloned_from_ptr(ptr: *const Self) -> Arc<Self> {
        #[allow(clippy::disallowed_methods)]
        Arc::increment_strong_count(ptr);
        #[allow(clippy::disallowed_methods)]
        Arc::from_raw(ptr)
    }
    unsafe fn as_maybe_ref<'a>(ptr: *const Self) -> Option<&'a Self> {
        #[allow(clippy::disallowed_methods)]
        ptr.as_ref()
    }
    unsafe fn as_ref<'a>(ptr: *const Self) -> &'a Self {
        #[allow(clippy::disallowed_methods)]
        ptr.as_ref().unwrap()
    }
    unsafe fn free(ptr: *const Self) {
        std::mem::drop(ArcFFI::from_ptr(ptr));
    }
}

impl ArcFFI for CassFuture {}

impl CassFuture {
    pub fn make_raw(
        fut: impl Future<Output = CassFutureResult> + Send + 'static,
    ) -> *mut CassFuture {
        Self::new_from_future(fut).into_raw() as *mut _
    }

    pub fn new_from_future(
        fut: impl Future<Output = CassFutureResult> + Send + 'static,
    ) -> Arc<CassFuture> {
        let cass_fut = Arc::new(CassFuture {
            state: Mutex::new(Default::default()),
            wait_for_value: Condvar::new(),
        });
        let cass_fut_clone = cass_fut.clone();
        let join_handle = RUNTIME.spawn(async move {
            let r = fut.await;
            let maybe_cb = {
                let mut guard = cass_fut_clone.state.lock().unwrap();
                guard.value = Some(r);
                // Take the callback and call it after releasing the lock
                guard.callback.take()
            };
            if let Some(bound_cb) = maybe_cb {
                bound_cb.invoke(cass_fut_clone.as_ref());
            }

            cass_fut_clone.wait_for_value.notify_all();
        });
        {
            let mut lock = cass_fut.state.lock().unwrap();
            lock.join_handle = Some(join_handle);
        }
        cass_fut
    }

    fn into_raw(self: Arc<Self>) -> *const Self {
        ArcFFI::into_ptr(self)
    }

    pub fn with_waited_result<T>(&self, f: impl FnOnce(&mut CassFutureResult) -> T) -> T {
        self.with_waited_state(|s| f(s.value.as_mut().unwrap()))
    }

    /// Awaits the future until completion.
    ///
    /// There are two possible cases:
    /// - noone is currently working on the future -> we take the ownership
    ///   of JoinHandle (future) and we poll it until completion.
    /// - some other thread is working on the future -> we wait on the condition
    ///   variable to get an access to the future's state. Once we are notified,
    ///   there are two cases:
    ///     - JoinHandle is consumed -> some other thread already resolved the future.
    ///       We can return.
    ///     - JoinHandle is Some -> some other thread was working on the future, but
    ///       timed out (see [CassFuture::with_waited_state_timed]). We need to
    ///       take the ownership of the handle, and complete the work.
    fn with_waited_state<T>(&self, f: impl FnOnce(&mut CassFutureState) -> T) -> T {
        let mut guard = self.state.lock().unwrap();
        loop {
            let handle = guard.join_handle.take();
            if let Some(handle) = handle {
                mem::drop(guard);
                // unwrap: JoinError appears only when future either panic'ed or canceled.
                RUNTIME.block_on(handle).unwrap();
                guard = self.state.lock().unwrap();
            } else {
                guard = self
                    .wait_for_value
                    .wait_while(guard, |state| {
                        state.value.is_none() && state.join_handle.is_none()
                    })
                    // unwrap: Error appears only when mutex is poisoned.
                    .unwrap();
                if guard.join_handle.is_some() {
                    // join_handle was none, and now it isn't - some other thread must
                    // have timed out and returned the handle. We need to take over
                    // the work of completing the future. To do that, we go into
                    // another iteration so that we land in the branch with block_on.
                    continue;
                }
            }
            return f(&mut guard);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cass_future_wait(future_raw: *const CassFuture) {
    ArcFFI::as_ref(future_raw).with_waited_result(|_| ());
}

#[no_mangle]
pub unsafe extern "C" fn cass_future_ready(future_raw: *const CassFuture) -> bool {
    let state_guard = ArcFFI::as_ref(future_raw).state.lock().unwrap();
    match state_guard.value {
        None => false,
        Some(_) => true,
    }
}

#[no_mangle]
pub extern "C" fn cass_future_debug_info(future: *const CassFuture) -> *const c_char {
    if future.is_null() {
        return std::ptr::null();
    }

    let future = unsafe { &*future };
    let state = future.state.lock().unwrap();

    // Format the CassFutureResult
    let result_info = match &state.value {
        None => "No result yet".to_string(),
        Some(Ok(CassResultValue::Empty)) => "Result is empty".to_string(),
        Some(Ok(CassResultValue::QueryResult(value))) => format!("Query Result: {}", value),
        Some(Ok(CassResultValue::QueryError(err))) => format!("Query Error: {}", err),
        Some(Err(err)) => format!("Error: {:?}", err),
    };

    // Include the error string (if any)
    let error_info = state.err_string.as_deref().unwrap_or("No error");

    // Combine the debug information
    let debug_info = format!(
        "Result: {}, Error: {}",
        result_info, error_info
    );

    CString::new(debug_info).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn cass_future_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            CString::from_raw(s); // Reclaims memory allocated by `CString::into_raw`
        }
    }
}
