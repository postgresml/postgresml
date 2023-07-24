use std::ffi::CStr;

#[cfg(any(test, feature = "pg_test"))]
use pgrx::{pg_schema, pg_test};
use pgrx_pg_sys::AsPgCStr;

pub fn get_config(name: &str) -> Option<String> {
    // SAFETY: name is not null because it is a Rust reference.
    let ptr = unsafe { pgrx_pg_sys::GetConfigOption(name.as_pg_cstr(), true, false) };
    (!ptr.is_null()).then(move || {
        // SAFETY: assuming pgrx_pg_sys is providing a valid, null terminated pointer.
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string()
    })
}

#[cfg(any(test, feature = "pg_test"))]
pub fn set_config(name: &str, value: &str) -> Result<(), pgrx::spi::Error> {
    // using Spi::run instead of pgrx_pg_sys interface because it seems much easier,
    // especially since this is just for testing
    let query = format!("SELECT set_config('{name}', '{value}', false);");
    pgrx::Spi::run(&query)
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn read_config_max_connections() {
        let name = "max_connections";
        assert_eq!(get_config(name), Some("100".into()));
    }

    #[pg_test]
    fn read_pgml_huggingface_whitelist() {
        let name = "pgml.huggingface_whitelist";
        let value = "meta-llama/Llama-2-7b";
        set_config(name, value).unwrap();
        assert_eq!(get_config(name), Some(value.into()));
    }
}
