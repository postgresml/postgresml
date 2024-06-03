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
    };
}

gen_custom_into!(());
gen_custom_into!(bool);

unsafe impl<T1, T2: CustomInto<T1>> CustomInto<Vec<T1>> for Vec<T2> {
    unsafe fn custom_into(self) -> Vec<T1> {
        self.into_iter().map(|x| x.custom_into()).collect()
    }
}
