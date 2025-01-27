use std::sync::Arc;

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
    // unsafe fn cloned_from_ptr(ptr: *const Self) -> Arc<Self> {
    //     #[allow(clippy::disallowed_methods)]
    //     Arc::increment_strong_count(ptr);
    //     #[allow(clippy::disallowed_methods)]
    //     Arc::from_raw(ptr)
    // }
    // unsafe fn as_maybe_ref<'a>(ptr: *const Self) -> Option<&'a Self> {
    //     #[allow(clippy::disallowed_methods)]
    //     ptr.as_ref()
    // }
    // unsafe fn as_ref<'a>(ptr: *const Self) -> &'a Self {
    //     #[allow(clippy::disallowed_methods)]
    //     ptr.as_ref().unwrap()
    // }
    unsafe fn free(ptr: *const Self) {
        std::mem::drop(ArcFFI::from_ptr(ptr));
    }
}
