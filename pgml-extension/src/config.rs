use std::ffi::CStr;

#[cfg(any(test, feature = "pg_test"))]
use pgrx::{pg_schema, pg_test};
use pgrx_pg_sys::AsPgCStr;

pub fn read_config(name: &str) -> Option<String> {
    // SAFETY: name is not null because it is a Rust reference.
    let ptr = unsafe { pgrx_pg_sys::GetConfigOption(name.as_pg_cstr(), true, false) };
    (!ptr.is_null()).then(move || {
        // SAFETY: assuming pgrx_pg_sys is providing a valid, null terminated pointer.
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string()
    })
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn read_config_max_connections() {
        let name = "max_connections";
        assert_eq!(read_config(name), Some("100".into()));
    }
}
