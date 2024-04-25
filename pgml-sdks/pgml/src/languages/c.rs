use crate::types::{DateTime, GeneralJsonAsyncIterator, GeneralJsonIterator, Json};
use futures::pin_mut;
use futures::stream::Stream;
use rust_bridge::c::CustomInto;
use std::pin::Pin;

pub type JsonC = std::ffi::c_char;

unsafe impl CustomInto<Json> for *mut JsonC {
    unsafe fn custom_into(self) -> Json {
        let s = std::ffi::CStr::from_ptr(self).to_str().unwrap();
        eprintln!("\nABOU TO DECODE: {}\n", s);
        serde_json::from_str::<serde_json::Value>(s).unwrap().into()
    }
}

unsafe impl CustomInto<*mut JsonC> for Json {
    unsafe fn custom_into(self) -> *mut JsonC {
        let s = serde_json::to_string(&self).unwrap();
        std::ffi::CString::new(s).unwrap().into_raw()
    }
}

#[repr(C)]
pub struct GeneralJsonIteratorC {
    pub wrapped:
        *mut std::iter::Peekable<Box<dyn Iterator<Item = Result<Json, anyhow::Error>> + Send>>,
}

unsafe impl CustomInto<*mut GeneralJsonIteratorC> for GeneralJsonIterator {
    unsafe fn custom_into(self) -> *mut GeneralJsonIteratorC {
        Box::into_raw(Box::new(GeneralJsonIteratorC {
            wrapped: Box::into_raw(Box::new(self.0.peekable())),
        }))
    }
}

#[no_mangle]
pub unsafe extern "C" fn GeneralJsonIteratorC_done(iterator: *mut GeneralJsonIteratorC) -> bool {
    let mut c = Box::leak(Box::from_raw(iterator));
    if let Some(_) = (*c.wrapped).peek() {
        false
    } else {
        true
    }
}

#[no_mangle]
pub unsafe extern "C" fn GeneralJsonIteratorC_next(
    iterator: *mut GeneralJsonIteratorC,
) -> *mut JsonC {
    let c = Box::leak(Box::from_raw(iterator));
    let b = Box::leak(Box::from_raw(c.wrapped));
    (*b).next().unwrap().unwrap().custom_into()
}

#[repr(C)]
pub struct GeneralJsonAsyncIteratorC {
    pub wrapped: *mut futures::stream::Peekable<
        Pin<Box<dyn Stream<Item = Result<Json, anyhow::Error>> + Send>>,
    >,
}

unsafe impl CustomInto<*mut GeneralJsonAsyncIteratorC> for GeneralJsonAsyncIterator {
    unsafe fn custom_into(self) -> *mut GeneralJsonAsyncIteratorC {
        use futures::stream::StreamExt;
        Box::into_raw(Box::new(GeneralJsonAsyncIteratorC {
            wrapped: Box::into_raw(Box::new(self.0.peekable())),
        }))
    }
}

#[no_mangle]
pub unsafe extern "C" fn GeneralJsonAsyncIteratorC_done(
    iterator: *mut GeneralJsonAsyncIteratorC,
) -> bool {
    crate::get_or_set_runtime().block_on(async move {
        use futures::stream::StreamExt;
        let c = Box::leak(Box::from_raw(iterator));
        let s = Box::leak(Box::from_raw(c.wrapped));
        let mut s = Pin::new(s);
        let res = s.as_mut().peek_mut().await;
        if let Some(res) = res {
            false
        } else {
            true
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn GeneralJsonAsyncIteratorC_next(
    iterator: *mut GeneralJsonAsyncIteratorC,
) -> *mut JsonC {
    crate::get_or_set_runtime().block_on(async move {
        use futures::stream::StreamExt;
        let mut c = Box::leak(Box::from_raw(iterator));
        (*c.wrapped).next().await.unwrap().unwrap().custom_into()
    })
}
