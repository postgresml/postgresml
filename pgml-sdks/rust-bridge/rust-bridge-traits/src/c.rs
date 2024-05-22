/// Very similar to the `Into` trait, but we can implement it on foreign types.
pub unsafe trait CustomInto<T> {
    unsafe fn custom_into(self) -> T;
}

pub unsafe trait CustomIntoVec<T> {
    unsafe fn custom_into_vec(self, size: usize) -> Vec<T>;
}

unsafe impl<T1, T2> CustomIntoVec<T1> for *mut *mut T2
where
    *mut T2: CustomInto<T1>,
{
    unsafe fn custom_into_vec(self, size: usize) -> Vec<T1> {
        let mut result = vec![];
        let strings = std::slice::from_raw_parts_mut(self, size);
        for s in strings {
            let res = s.custom_into();
            result.push(res)
        }
        result
    }
}

unsafe impl<'a> CustomInto<&'a str> for *mut std::ffi::c_char {
    unsafe fn custom_into(self) -> &'a str {
        std::ffi::CStr::from_ptr(self).to_str().unwrap()
    }
}

unsafe impl CustomInto<String> for *mut std::ffi::c_char {
    unsafe fn custom_into(self) -> String {
        std::ffi::CStr::from_ptr(self).to_str().unwrap().to_string()
    }
}

unsafe impl CustomInto<*mut std::ffi::c_char> for String {
    unsafe fn custom_into(self) -> *mut std::ffi::c_char {
        std::ffi::CString::new(self).unwrap().into_raw()
    }
}

unsafe impl CustomInto<i32> for *mut std::ffi::c_int {
    unsafe fn custom_into(self) -> i32 {
        *self
    }
}

unsafe impl CustomInto<f64> for *mut std::ffi::c_double {
    unsafe fn custom_into(self) -> f64 {
        *self
    }
}

unsafe impl<T1, T2> CustomInto<Option<T1>> for *mut T2
where
    *mut T2: CustomInto<T1>,
{
    unsafe fn custom_into(self) -> Option<T1> {
        if self.is_null() {
            None
        } else {
            Some(self.custom_into())
        }
    }
}

unsafe impl<T1, T2> CustomInto<(*mut T1, usize)> for Vec<T2>
where
    T2: CustomInto<T1>,
{
    unsafe fn custom_into(self) -> (*mut T1, usize) {
        let size = self.len();
        let v: Vec<T1> = self.into_iter().map(|v| v.custom_into()).collect();
        (v.leak().as_mut_ptr(), size)
    }
}

macro_rules! gen_custom_into {
    ($t1:ty) => {
        unsafe impl CustomInto<$t1> for $t1 {
            unsafe fn custom_into(self) -> $t1 {
                self
            }
        }
    }; // (($($T1:ident),+), ($($T2:ident),+), ($($C:tt),+)) => {
       //     impl<$($T1, $T2: CustomInto<$T1>),+> CustomInto<($($T1),+,)> for ($($T2),+,) {
       //         fn custom_into(self) -> ($($T1),+,) {
       //             ($(self.$C.custom_into()),+,)
       //         }
       //     }
       // }
}

gen_custom_into!(());
gen_custom_into!(bool);

// impl<T1, T2: CustomInto<T1>> CustomInto<Option<T1>> for Option<T2> {
//     fn custom_into(self) -> Option<T1> {
//         self.map(|s| s.custom_into())
//     }
// }

unsafe impl<T1, T2: CustomInto<T1>> CustomInto<Vec<T1>> for Vec<T2> {
    unsafe fn custom_into(self) -> Vec<T1> {
        self.into_iter().map(|x| x.custom_into()).collect()
    }
}

// impl<K1: std::cmp::Eq + std::hash::Hash, T1, K2: CustomInto<K1>, T2: CustomInto<T1>>
//     CustomInto<HashMap<K1, T1>> for HashMap<K2, T2>
// {
//     fn custom_into(self) -> HashMap<K1, T1> {
//         self.into_iter()
//             .map(|(k, v)| (k.custom_into(), v.custom_into()))
//             .collect()
//     }
// }

// impl CustomInto<&'static str> for &str {
//     fn custom_into(self) -> &'static str {
//         // This is how we get around the liftime checker
//         unsafe {
//             let ptr = self as *const str;
//             let ptr = ptr as *mut str;
//             let boxed = Box::from_raw(ptr);
//             Box::leak(boxed)
//         }
//     }
// }

// gen_custom_into!((T1), (TT2), (0));
// gen_custom_into!((T1, T2), (TT1, TT2), (0, 1));
// gen_custom_into!((T1, T2, T3), (TT1, TT2, TT3), (0, 1, 2));
// gen_custom_into!((T1, T2, T3, T4), (TT1, TT2, TT3, TT4), (0, 1, 2, 3));
// gen_custom_into!(
//     (T1, T2, T3, T4, T5),
//     (TT1, TT2, TT3, TT4, TT5),
//     (0, 1, 2, 3, 4)
// );
// gen_custom_into!(
//     (T1, T2, T3, T4, T5, T6),
//     (TT1, TT2, TT3, TT4, TT5, TT6),
//     (0, 1, 2, 3, 4, 5)
// );

// // There are some restrictions I cannot figure out around conflicting trait
// // implimentations so this is my solution for now
// gen_custom_into!(String);

// gen_custom_into!(());

// gen_custom_into!(bool);

// gen_custom_into!(i8);
// gen_custom_into!(i16);
// gen_custom_into!(i32);
// gen_custom_into!(i64);

// gen_custom_into!(u8);
// gen_custom_into!(u16);
// gen_custom_into!(u32);
// gen_custom_into!(u64);

// gen_custom_into!(f32);
// gen_custom_into!(f64);
