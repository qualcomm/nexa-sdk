use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Error codes mirroring ml_ErrorCode from ml.h.
pub const ML_SUCCESS: i32 = 0;
pub const ML_ERROR_COMMON_UNKNOWN: i32 = -100000;
pub const ML_ERROR_COMMON_INVALID_INPUT: i32 = -100001;
pub const ML_ERROR_COMMON_NOT_INITIALIZED: i32 = -100007;
pub const ML_ERROR_COMMON_FILE_NOT_FOUND: i32 = -100004;

use model_manager_core::error::Error;

pub fn err_to_code(e: &Error) -> i32 {
    match e {
        Error::NotInitialized => ML_ERROR_COMMON_NOT_INITIALIZED,
        Error::ModelNotFound(_) => ML_ERROR_COMMON_FILE_NOT_FOUND,
        Error::QuantNotFound(_, _) | Error::QuantNotDownloaded(_, _) => {
            ML_ERROR_COMMON_INVALID_INPUT
        }
        Error::NoDownloadedQuant(_) => ML_ERROR_COMMON_INVALID_INPUT,
        _ => ML_ERROR_COMMON_UNKNOWN,
    }
}

/// Convert a raw C string pointer to a &str. Returns None if ptr is null or invalid UTF-8.
pub unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        None
    } else {
        CStr::from_ptr(ptr).to_str().ok()
    }
}

/// Allocate a CString from a Rust &str and return its raw pointer.
/// The caller is responsible for freeing it via ml_model_free_string().
pub fn str_to_cptr(s: &str) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

/// Free a CString pointer allocated by str_to_cptr.
pub unsafe fn free_cptr(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}
