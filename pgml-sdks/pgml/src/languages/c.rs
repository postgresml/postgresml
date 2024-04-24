use crate::types::{DateTime, GeneralJsonAsyncIterator, GeneralJsonIterator, Json};
use rust_bridge::c::CustomInto;

pub type JsonC = std::ffi::c_char;

unsafe impl CustomInto<Json> for *mut JsonC {
    unsafe fn custom_into(self) -> Json {
        let s = std::ffi::CStr::from_ptr(self).to_str().unwrap();
        serde_json::from_str::<serde_json::Value>(s).unwrap().into()
    }
}

unsafe impl CustomInto<*mut JsonC> for Json {
    unsafe fn custom_into(self) -> *mut JsonC {
        let s = serde_json::to_string(&self).unwrap();
        std::ffi::CString::new(s).unwrap().into_raw()
    }
}
