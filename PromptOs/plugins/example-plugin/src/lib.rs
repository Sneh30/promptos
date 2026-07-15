//! Example PromptOS plugin demonstrating the plugin API.
//! This plugin adds a custom optimization pass that wraps instructions
//! in `<focus>` tags for improved model attention.

use std::slice;

#[no_mangle]
pub extern "C" fn __promptos_plugin_info() -> *const u8 {
    let info = r#"{"name": "my-optimizer", "version": "1.0.0"}"#;
    info.as_ptr()
}

#[no_mangle]
pub extern "C" fn __promptos_on_compile(prompt_ast: *const u8, len: u32) -> *const u8 {
    let input = unsafe {
        if prompt_ast.is_null() || len == 0 {
            return std::ptr::null();
        }
        let slice = slice::from_raw_parts(prompt_ast, len as usize);
        String::from_utf8_lossy(slice).to_string()
    };

    let output = format!("<focus>\n{}\n</focus>", input);
    let bytes = output.into_bytes();
    let boxed = bytes.into_boxed_slice();
    Box::into_raw(boxed) as *const u8
}

#[no_mangle]
pub extern "C" fn __promptos_on_diagnostic(_diag: *const u8, _len: u32) {
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_compile_empty() {
        let result = __promptos_on_compile(std::ptr::null(), 0);
        assert!(result.is_null());
    }
}
