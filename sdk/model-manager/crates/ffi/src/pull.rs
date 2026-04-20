use std::os::raw::{c_char, c_void};
use std::path::PathBuf;

use model_manager_core::hub::HubSource;
use model_manager_core::pull::{pull, PullRequest};
use crate::init::get_store;
use crate::types::*;

/// C-compatible hub source enum.
#[repr(C)]
#[allow(dead_code)]
pub enum MlHubSource {
    Auto = 0,
    HuggingFace = 1,
    ModelScope = 2,
    S3 = 3,
    Volces = 4,
    LocalFs = 5,
}

/// Progress callback type: (downloaded, total, user_data) → bool (false = cancel).
pub type MlDownloadProgressCb =
    Option<unsafe extern "C" fn(i64, i64, *mut c_void) -> bool>;

#[repr(C)]
pub struct MlModelPullInput {
    pub model_name: *const c_char,
    pub quant: *const c_char,
    pub hub: MlHubSource,
    pub local_path: *const c_char,
    pub on_progress: MlDownloadProgressCb,
    pub user_data: *mut c_void,
}

#[no_mangle]
pub extern "C" fn ml_model_pull(input: *const MlModelPullInput) -> i32 {
    if input.is_null() {
        return ML_ERROR_COMMON_INVALID_INPUT;
    }

    let inp = unsafe { &*input };

    let model_name = match unsafe { cstr_to_str(inp.model_name) } {
        Some(s) => s.to_string(),
        None => return ML_ERROR_COMMON_INVALID_INPUT,
    };

    let hub = match inp.hub {
        MlHubSource::HuggingFace | MlHubSource::Auto => HubSource::HuggingFace,
        MlHubSource::LocalFs => {
            let path = match unsafe { cstr_to_str(inp.local_path) } {
                Some(s) => PathBuf::from(s),
                None => return ML_ERROR_COMMON_INVALID_INPUT,
            };
            HubSource::LocalFs(path)
        }
        // Other hubs not yet implemented; fall back to HuggingFace.
        _ => HubSource::HuggingFace,
    };

    // Wrap C callback into a Rust closure.
    // `*mut c_void` is not Send/Sync, so we wrap both the callback and the user_data
    // in a newtype that asserts thread-safety.  The caller is responsible for ensuring
    // both values remain valid and the callback is safe to call for the duration of
    // this blocking pull operation.
    struct CCallback {
        cb: unsafe extern "C" fn(i64, i64, *mut c_void) -> bool,
        user_data: *mut c_void,
    }
    unsafe impl Send for CCallback {}
    unsafe impl Sync for CCallback {}

    let progress_cb: Option<model_manager_core::hub::ProgressCallback> =
        if let Some(cb) = inp.on_progress {
            let cc = CCallback { cb, user_data: inp.user_data };
            Some(Box::new(move |downloaded: i64, total: i64| -> bool {
                unsafe { (cc.cb)(downloaded, total, cc.user_data) }
            }))
        } else {
            None
        };

    let store = match get_store() {
        Ok(s) => s,
        Err(c) => return c,
    };

    let req = PullRequest { model_name, hub, on_progress: progress_cb };

    match pull(store, req) {
        Ok(()) => ML_SUCCESS,
        Err(e) => {
            eprintln!("[model-manager] ml_model_pull error: {e}");
            err_to_code(&e)
        }
    }
}
