use crate::compiler::LlamaCompiler;
use crate::download::ModelDownloader;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

static COMPILER: once_cell::sync::Lazy<Mutex<Option<LlamaCompiler>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

#[no_mangle]
pub extern "C" fn promptos_llm_init(model_path: *const c_char) -> i32 {
    let path = unsafe {
        if model_path.is_null() {
            return -1;
        }
        match CStr::from_ptr(model_path).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return -2,
        }
    };

    let compiler = LlamaCompiler::new(&path);
    match compiler.load_model() {
        Ok(_) => {
            let mut guard = COMPILER.lock().unwrap();
            *guard = Some(compiler);
            0
        }
        Err(e) => {
            tracing::error!("Failed to load model: {}", e);
            -3
        }
    }
}

#[no_mangle]
pub extern "C" fn promptos_llm_is_loaded() -> i32 {
    let guard = COMPILER.lock().unwrap();
    match guard.as_ref() {
        Some(c) if c.is_model_loaded() => 1,
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn promptos_llm_compile(
    input: *const c_char,
    output: *mut c_char,
    output_max_len: i32,
) -> i32 {
    let input_str = unsafe {
        if input.is_null() {
            return -1;
        }
        match CStr::from_ptr(input).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return -2,
        }
    };

    let guard = COMPILER.lock().unwrap();
    let compiler = match guard.as_ref() {
        Some(c) => c,
        None => return -3,
    };

    match compiler.compile(&input_str) {
        Ok(result) => {
            let out = if result.optimized_text.is_empty() {
                input_str
            } else {
                result.optimized_text
            };
            match CString::new(out) {
                Ok(cstr) => {
                    let bytes = cstr.as_bytes_with_nul();
                    let len = bytes.len();
                    let max = output_max_len as usize;
                    if len > max {
                        let truncated = &bytes[..max - 1];
                        unsafe {
                            std::ptr::copy_nonoverlapping(truncated.as_ptr(), output as *mut u8, truncated.len());
                            *output.add(truncated.len()) = 0;
                        }
                        0
                    } else {
                        unsafe {
                            std::ptr::copy_nonoverlapping(bytes.as_ptr(), output as *mut u8, len);
                        }
                        0
                    }
                }
                Err(_) => -5,
            }
        }
        Err(_) => -4,
    }
}

#[no_mangle]
pub extern "C" fn promptos_llm_unload() -> i32 {
    let mut guard = COMPILER.lock().unwrap();
    *guard = None;
    0
}

#[no_mangle]
pub extern "C" fn promptos_llm_download_model(model_path: *mut c_char, max_len: i32) -> i32 {
    let downloader = ModelDownloader::new();

    if downloader.is_model_downloaded() {
        let path = downloader.default_model_path();
        let path_str = path.to_string_lossy().to_string();
        if let Ok(cstr) = CString::new(path_str) {
            let bytes = cstr.as_bytes_with_nul();
            let len = bytes.len();
            let max = max_len as usize;
            if len <= max {
                unsafe {
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), model_path as *mut u8, len);
                }
            }
        }
        return 1;
    }

    match downloader.download_default_model() {
        Ok(path) => {
            if let Ok(cstr) = CString::new(path) {
                let bytes = cstr.as_bytes_with_nul();
                let len = bytes.len();
                let max = max_len as usize;
                if len <= max {
                    unsafe {
                        std::ptr::copy_nonoverlapping(bytes.as_ptr(), model_path as *mut u8, len);
                    }
                }
            }
            0
        }
        Err(e) => {
            tracing::error!("Download failed: {}", e);
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn promptos_llm_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}
