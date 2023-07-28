#[cfg(feature = "javascript")]
pub mod javascript;

#[cfg(feature = "python")]
pub mod python;

use std::collections::HashMap;

use crate::types::{Json, DateTime};

macro_rules! gen_custom_into_for_self {
    ($t1:ty) => {
        impl CustomInto<$t1> for $t1 {
           fn custom_into(self) -> $t1 {
               self
           } 
        }
    }
}

/// Very similar to the `Into` trait, but we can implement it on foreign types
pub trait CustomInto<T> {
    fn custom_into(self) -> T;
}

impl<T1, T2: CustomInto<T1>> CustomInto<Option<T1>> for Option<T2> {
    fn custom_into(self) -> Option<T1> {
        match self {
            Some(s) => Some(s.custom_into()),
            None => None,
        }
    }
}

impl<T1, T2: CustomInto<T1>> CustomInto<Vec<T1>> for Vec<T2> {
    fn custom_into(self) -> Vec<T1> {
        self.into_iter().map(|x| x.custom_into()).collect()
    }
}

impl<K1: std::cmp::Eq + std::hash::Hash, T1, K2: CustomInto<K1>, T2: CustomInto<T1>> CustomInto<HashMap<K1, T1>> for HashMap<K2, T2> {
    fn custom_into(self) -> HashMap<K1, T1> {
        self.into_iter().map(|(k, v)| (k.custom_into(), v.custom_into())).collect()
    }
}

impl CustomInto<&'static str> for &str {
    fn custom_into(self) -> &'static str {
        // This is how we get around the liftime checker
        unsafe {
            let ptr = &*self as *const str;
            let ptr = ptr as *mut str;
            let boxed = Box::from_raw(ptr);
            Box::leak(boxed)
        }
    }
}

// There are some really dumb restrictions I cannot figure out around conflicting trait
// implimentations so this is my solution for now
gen_custom_into_for_self!(String);

gen_custom_into_for_self!(bool);

gen_custom_into_for_self!(Json);
gen_custom_into_for_self!(DateTime);

gen_custom_into_for_self!(i8);
gen_custom_into_for_self!(i16);
gen_custom_into_for_self!(i32);
gen_custom_into_for_self!(i64);

gen_custom_into_for_self!(u8);
gen_custom_into_for_self!(u16);
gen_custom_into_for_self!(u32);
gen_custom_into_for_self!(u64);

gen_custom_into_for_self!(f32);
gen_custom_into_for_self!(f64);
