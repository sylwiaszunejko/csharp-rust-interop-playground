use std::ffi::{CString, CStr};
use std::os::raw::c_char;

/// Converts a Rust String into a *const c_char (C-style string).
pub (crate) fn string_to_c_char(s: &str) -> *const c_char {
    let c_string = CString::new(s).expect("CString::new failed");
    c_string.into_raw()
}

/// Converts a *const c_char (C-style string) back into a Rust String.
pub (crate) unsafe fn c_char_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let c_str = CStr::from_ptr(ptr);
    c_str.to_string_lossy().into_owned()
}

/// Frees the memory allocated by CString::into_raw()
pub (crate) unsafe fn free_c_char(ptr: *const c_char) {
    if ptr.is_null() {
        return;
    }
    drop(CString::from_raw(ptr as *mut c_char));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_c_char_and_back() {
        let original = "Hello, Rust!";
        let c_char_ptr = string_to_c_char(original);

        unsafe {
            let converted = c_char_to_string(c_char_ptr);
            assert_eq!(original, converted, "Conversion failed");
            free_c_char(c_char_ptr);
        }
    }

    #[test]
    fn test_empty_string_conversion() {
        let original = "";
        let c_char_ptr = string_to_c_char(original);

        unsafe {
            let converted = c_char_to_string(c_char_ptr);
            assert_eq!(original, converted, "Empty string conversion failed");
            free_c_char(c_char_ptr);
        }
    }

    #[test]
    fn test_null_pointer_conversion() {
        unsafe {
            let converted = c_char_to_string(std::ptr::null());
            assert_eq!(converted, "", "Null pointer should return an empty string");
        }
    }
}
