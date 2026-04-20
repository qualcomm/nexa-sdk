use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::OnceLock;

use model_manager_core::{config::StoreConfig, store::Store};
use crate::types::*;

static STORE: OnceLock<Store> = OnceLock::new();

/// Access the global store; returns an Err if not yet initialized.
pub(crate) fn get_store() -> Result<&'static Store, i32> {
    STORE.get().ok_or(ML_ERROR_COMMON_NOT_INITIALIZED)
}

/// Initialize the model manager.
/// `data_dir` may be NULL (defaults to `~/.cache/geniex`).
/// `hf_token` may be NULL (anonymous HF access).
#[no_mangle]
pub extern "C" fn ml_model_init(
    data_dir: *const c_char,
    hf_token: *const c_char,
) -> i32 {
    let data_dir_path = unsafe {
        cstr_to_str(data_dir)
            .map(PathBuf::from)
    };
    let token = unsafe { cstr_to_str(hf_token).map(str::to_string) };

    let cfg = if let Some(dir) = data_dir_path {
        StoreConfig::new(dir, token)
    } else {
        let mut c = StoreConfig::from_env();
        if let Some(t) = token {
            c.hf_token = Some(t);
        }
        c
    };

    let store = match Store::new(cfg) {
        Ok(s) => s,
        Err(e) => return err_to_code(&e),
    };

    // OnceLock: if already initialized we silently succeed.
    let _ = STORE.set(store);
    ML_SUCCESS
}

/// Deinitialize the model manager. No-op in this implementation (Store has no background threads).
#[no_mangle]
pub extern "C" fn ml_model_deinit() -> i32 {
    ML_SUCCESS
}
