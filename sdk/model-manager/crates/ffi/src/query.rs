use std::os::raw::c_char;
use std::path::PathBuf;

use model_manager_core::manifest_builder::ManifestHint;
use model_manager_core::mapping::canonicalize_model_name;
use model_manager_core::pull::PullRequest;
use model_manager_core::query::query_blocking;

use crate::init::{get_store, runtime_handle};
use crate::pull::{build_pull_intent, GenieXHubSource};
use crate::store::{to_ffi_type, GenieXModelType};
use crate::types::*;

/// Input for a plan-only query. Mirrors `geniex_ModelQueryInput` in
/// geniex_model.h — a download-free subset of `geniex_ModelPullInput`
/// (no quant hint, no progress callback).
#[repr(C)]
pub struct GenieXModelQueryInput {
    pub struct_size: u32,
    pub model_name: *const c_char,
    pub hub: GenieXHubSource,
    pub local_path: *const c_char,
    pub hf_token: *const c_char,
    pub chipset: *const c_char,
    pub display_name: *const c_char,
}

const ACCEPTED_QUERY_INPUT_SIZES: &[u32] = &[std::mem::size_of::<GenieXModelQueryInput>() as u32];

/// One advertised quantization. Mirrors `geniex_QuantCandidate`.
#[repr(C)]
pub struct GenieXQuantCandidate {
    pub quant: *mut c_char,
    pub size: i64,
    pub is_default: bool,
}

/// Result of `geniex_model_query`. Mirrors `geniex_ModelQueryOutput`.
#[repr(C)]
pub struct GenieXModelQueryOutput {
    pub model_name: *mut c_char,
    pub plugin_id: *mut c_char,
    pub model_type: GenieXModelType,
    pub candidates: *mut GenieXQuantCandidate,
    pub candidate_count: i32,
}

impl GenieXModelQueryOutput {
    fn null() -> Self {
        Self {
            model_name: std::ptr::null_mut(),
            plugin_id: std::ptr::null_mut(),
            model_type: GenieXModelType::Llm,
            candidates: std::ptr::null_mut(),
            candidate_count: 0,
        }
    }
}

#[no_mangle]
pub extern "C" fn geniex_model_query(
    input: *const GenieXModelQueryInput,
    out: *mut GenieXModelQueryOutput,
) -> i32 {
    ffi_guard(|| {
        if input.is_null() || out.is_null() {
            return GENIEX_ERROR_COMMON_INVALID_INPUT;
        }

        // ABI gate, read before any other field (see geniex_model_pull).
        let struct_size = unsafe { std::ptr::read(&(*input).struct_size) };
        if !ACCEPTED_QUERY_INPUT_SIZES.contains(&struct_size) {
            crate::logging::error(&format!(
                "geniex_model_query: unsupported struct_size {}; expected one of {:?} \
                 (recompile your binding against the current geniex_model.h)",
                struct_size, ACCEPTED_QUERY_INPUT_SIZES,
            ));
            return GENIEX_ERROR_COMMON_INVALID_INPUT;
        }

        let inp = unsafe { &*input };

        let raw_model_name = match unsafe { cstr_to_str(inp.model_name) } {
            Some(s) => s.to_string(),
            None => return GENIEX_ERROR_COMMON_INVALID_INPUT,
        };
        let model_name = canonicalize_model_name(&raw_model_name);

        let hf_token = unsafe { cstr_to_str(inp.hf_token) }
            .map(str::to_string)
            .or_else(model_manager_core::config::StoreConfig::hf_token_from_env);
        let chipset = unsafe { cstr_to_str(inp.chipset) }
            .unwrap_or("")
            .to_string();
        let explicit_display_name = unsafe { cstr_to_str(inp.display_name) }
            .map(str::to_string)
            .filter(|s| !s.is_empty());
        let local_path = unsafe { cstr_to_str(inp.local_path) }.map(PathBuf::from);

        let intent = match build_pull_intent(
            &inp.hub,
            &model_name,
            hf_token,
            chipset,
            explicit_display_name,
            local_path,
        ) {
            Ok(i) => i,
            Err(c) => return c,
        };

        let store = match get_store() {
            Ok(s) => s,
            Err(c) => return c,
        };

        let req = PullRequest {
            model_name,
            intent,
            on_progress: None,
            hint: ManifestHint::default(),
        };

        let result = match query_blocking(&runtime_handle(), store, req) {
            Ok(r) => r,
            Err(e) => return report(&e),
        };

        let mut cands: Vec<GenieXQuantCandidate> = result
            .candidates
            .iter()
            .map(|c| GenieXQuantCandidate {
                quant: str_to_cptr(&c.quant),
                size: c.size,
                is_default: c.is_default,
            })
            .collect();
        cands.shrink_to_fit();
        let candidate_count = cands.len() as i32;
        let candidates = if cands.is_empty() {
            std::ptr::null_mut()
        } else {
            cands.as_mut_ptr()
        };
        std::mem::forget(cands);

        unsafe {
            (*out).model_name = str_to_cptr(&result.model_name);
            (*out).plugin_id = str_to_cptr(&result.plugin_id);
            (*out).model_type = to_ffi_type(result.model_type);
            (*out).candidates = candidates;
            (*out).candidate_count = candidate_count;
        }
        GENIEX_SUCCESS
    })
}

#[no_mangle]
pub unsafe extern "C" fn geniex_model_query_free(out: *mut GenieXModelQueryOutput) {
    if out.is_null() {
        return;
    }
    let o = &mut *out;
    free_cptr(o.model_name);
    free_cptr(o.plugin_id);
    if !o.candidates.is_null() {
        let slice = std::slice::from_raw_parts_mut(o.candidates, o.candidate_count as usize);
        for c in slice.iter_mut() {
            free_cptr(c.quant);
        }
        drop(Vec::from_raw_parts(
            o.candidates,
            o.candidate_count as usize,
            o.candidate_count as usize,
        ));
    }
    *out = GenieXModelQueryOutput::null();
}
