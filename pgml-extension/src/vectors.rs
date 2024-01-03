use pgrx::array::RawArray;
use pgrx::*;

#[pg_extern(immutable, parallel_safe, strict, name = "add")]
fn add_scalar_s(vector: Array<f32>, addend: f32) -> Vec<f32> {
    vector.iter_deny_null().map(|a| a + addend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "add")]
fn add_scalar_d(vector: Array<f64>, addend: f64) -> Vec<f64> {
    vector.iter_deny_null().map(|a| a + addend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "subtract")]
fn subtract_scalar_s(vector: Array<f32>, subtahend: f32) -> Vec<f32> {
    vector.iter_deny_null().map(|a| a - subtahend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "subtract")]
fn subtract_scalar_d(vector: Array<f64>, subtahend: f64) -> Vec<f64> {
    vector.iter_deny_null().map(|a| a - subtahend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "multiply")]
fn multiply_scalar_s(vector: Array<f32>, multiplicand: f32) -> Vec<f32> {
    vector.iter_deny_null().map(|a| a * multiplicand).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "multiply")]
fn multiply_scalar_d(vector: Array<f64>, multiplicand: f64) -> Vec<f64> {
    vector.iter_deny_null().map(|a| a * multiplicand).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "divide")]
fn divide_scalar_s(vector: Array<f32>, dividend: f32) -> Vec<f32> {
    vector.iter_deny_null().map(|a| a / dividend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "divide")]
fn divide_scalar_d(vector: Array<f64>, dividend: f64) -> Vec<f64> {
    vector.iter_deny_null().map(|a| a / dividend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "add")]
fn add_vector_s(vector: Array<f32>, addend: Array<f32>) -> Vec<f32> {
    vector
        .iter_deny_null()
        .zip(addend.iter_deny_null())
        .map(|(a, b)| a + b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "add")]
fn add_vector_d(vector: Array<f64>, addend: Array<f64>) -> Vec<f64> {
    vector
        .iter_deny_null()
        .zip(addend.iter_deny_null())
        .map(|(a, b)| a + b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "subtract")]
fn subtract_vector_s(vector: Array<f32>, subtahend: Array<f32>) -> Vec<f32> {
    vector
        .iter_deny_null()
        .zip(subtahend.iter_deny_null())
        .map(|(a, b)| a - b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "subtract")]
fn subtract_vector_d(vector: Array<f64>, subtahend: Array<f64>) -> Vec<f64> {
    vector
        .iter_deny_null()
        .zip(subtahend.iter_deny_null())
        .map(|(a, b)| a - b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "multiply")]
fn multiply_vector_s(vector: Array<f32>, multiplicand: Array<f32>) -> Vec<f32> {
    vector
        .iter_deny_null()
        .zip(multiplicand.iter_deny_null())
        .map(|(a, b)| a * b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "multiply")]
fn multiply_vector_d(vector: Array<f64>, multiplicand: Array<f64>) -> Vec<f64> {
    vector
        .iter_deny_null()
        .zip(multiplicand.iter_deny_null())
        .map(|(a, b)| a * b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "divide")]
fn divide_vector_s(vector: Array<f32>, dividend: Array<f32>) -> Vec<f32> {
    vector
        .iter_deny_null()
        .zip(dividend.iter_deny_null())
        .map(|(a, b)| a / b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "divide")]
fn divide_vector_d(vector: Array<f64>, dividend: Array<f64>) -> Vec<f64> {
    vector
        .iter_deny_null()
        .zip(dividend.iter_deny_null())
        .map(|(a, b)| a / b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_l0")]
fn norm_l0_s(vector: Array<f32>) -> f32 {
    vector.iter_deny_null().map(|a| if a == 0.0 { 0.0 } else { 1.0 }).sum()
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_l0")]
fn norm_l0_d(vector: Array<f64>) -> f64 {
    vector.iter_deny_null().map(|a| if a == 0.0 { 0.0 } else { 1.0 }).sum()
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_l1")]
fn norm_l1_s(vector: Array<f32>) -> f32 {
    unsafe {
        let vector: &[f32] = RawArray::from_array(vector).unwrap().data().as_ref();
        blas::sasum(vector.len().try_into().unwrap(), vector, 1)
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_l1")]
fn norm_l1_d(vector: Array<f64>) -> f64 {
    unsafe {
        let vector: &[f64] = RawArray::from_array(vector).unwrap().data().as_ref();
        blas::dasum(vector.len().try_into().unwrap(), vector, 1)
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_l2")]
fn norm_l2_s(vector: Array<f32>) -> f32 {
    unsafe {
        let vector: &[f32] = RawArray::from_array(vector).unwrap().data().as_ref();
        blas::snrm2(vector.len().try_into().unwrap(), vector, 1)
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_l2")]
fn norm_l2_d(vector: Array<f64>) -> f64 {
    unsafe {
        let vector: &[f64] = RawArray::from_array(vector).unwrap().data().as_ref();
        blas::dnrm2(vector.len().try_into().unwrap(), vector, 1)
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_max")]
fn norm_max_s(vector: Array<f32>) -> f32 {
    unsafe {
        let vector: &[f32] = RawArray::from_array(vector).unwrap().data().as_ref();
        let index = blas::isamax(vector.len().try_into().unwrap(), vector, 1);
        vector[index - 1].abs()
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_max")]
fn norm_max_d(vector: Array<f64>) -> f64 {
    unsafe {
        let vector: &[f64] = RawArray::from_array(vector).unwrap().data().as_ref();
        let index = blas::idamax(vector.len().try_into().unwrap(), vector, 1);
        vector[index - 1].abs()
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "normalize_l1")]
fn normalize_l1_s(vector: Array<f32>) -> Vec<f32> {
    let norm: f32;
    unsafe {
        let vector: &[f32] = RawArray::from_array(vector).unwrap().data().as_ref();
        norm = blas::sasum(vector.len().try_into().unwrap(), vector, 1);
        vector.iter().map(|a| a / norm).collect()
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "normalize_l1")]
fn normalize_l1_d(vector: Array<f64>) -> Vec<f64> {
    let norm: f64;
    unsafe {
        let vector: &[f64] = RawArray::from_array(vector).unwrap().data().as_ref();
        norm = blas::dasum(vector.len().try_into().unwrap(), vector, 1);
        vector.iter().map(|a| a / norm).collect()
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "normalize_l2")]
fn normalize_l2_s(vector: Array<f32>) -> Vec<f32> {
    let norm: f32;
    unsafe {
        let vector: &[f32] = RawArray::from_array(vector).unwrap().data().as_ref();
        norm = blas::snrm2(vector.len().try_into().unwrap(), vector, 1);
        vector.iter().map(|a| a / norm).collect()
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "normalize_l2")]
fn normalize_l2_d(vector: Array<f64>) -> Vec<f64> {
    let norm: f64;
    unsafe {
        let vector: &[f64] = RawArray::from_array(vector).unwrap().data().as_ref();
        norm = blas::dnrm2(vector.len().try_into().unwrap(), vector, 1);
        vector.iter().map(|a| a / norm).collect()
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "normalize_max")]
fn normalize_max_s(vector: Array<f32>) -> Vec<f32> {
    let norm;
    unsafe {
        let vector: &[f32] = RawArray::from_array(vector).unwrap().data().as_ref();
        let index = blas::isamax(vector.len().try_into().unwrap(), vector, 1);
        norm = vector[index - 1].abs();
        vector.iter().map(|a| a / norm).collect()
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "normalize_max")]
fn normalize_max_d(vector: Array<f64>) -> Vec<f64> {
    let norm;
    unsafe {
        let vector: &[f64] = RawArray::from_array(vector).unwrap().data().as_ref();
        let index = blas::idamax(vector.len().try_into().unwrap(), vector, 1);
        norm = vector[index - 1].abs();
        vector.iter().map(|a| a / norm).collect()
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "distance_l1")]
fn distance_l1_s(vector: Array<f32>, other: Array<f32>) -> f32 {
    vector
        .iter_deny_null()
        .zip(other.iter_deny_null())
        .map(|(a, b)| (a - b).abs())
        .sum()
}

#[pg_extern(immutable, parallel_safe, strict, name = "distance_l1")]
fn distance_l1_d(vector: Array<f64>, other: Array<f64>) -> f64 {
    vector
        .iter_deny_null()
        .zip(other.iter_deny_null())
        .map(|(a, b)| (a - b).abs())
        .sum()
}

#[pg_extern(immutable, parallel_safe, strict, name = "distance_l2")]
fn distance_l2_s(vector: Array<f32>, other: Array<f32>) -> f32 {
    vector
        .iter_deny_null()
        .zip(other.iter_deny_null())
        .map(|(a, b)| (a - b).powf(2.0))
        .sum::<f32>()
        .sqrt()
}

#[pg_extern(immutable, parallel_safe, strict, name = "distance_l2")]
fn distance_l2_d(vector: Array<f64>, other: Array<f64>) -> f64 {
    vector
        .iter_deny_null()
        .zip(other.iter_deny_null())
        .map(|(a, b)| (a - b).powf(2.0))
        .sum::<f64>()
        .sqrt()
}

#[pg_extern(immutable, parallel_safe, strict, name = "dot_product")]
fn dot_product_s(vector: Array<f32>, other: Array<f32>) -> f32 {
    unsafe {
        let vector: &[f32] = RawArray::from_array(vector).unwrap().data().as_ref();
        let other: &[f32] = RawArray::from_array(other).unwrap().data().as_ref();
        blas::sdot(vector.len().try_into().unwrap(), vector, 1, other, 1)
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "dot_product")]
fn dot_product_d(vector: Array<f64>, other: Array<f64>) -> f64 {
    unsafe {
        let vector: &[f64] = RawArray::from_array(vector).unwrap().data().as_ref();
        let other: &[f64] = RawArray::from_array(other).unwrap().data().as_ref();
        blas::ddot(vector.len().try_into().unwrap(), vector, 1, other, 1)
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "cosine_similarity")]
fn cosine_similarity_s(vector: Array<f32>, other: Array<f32>) -> f32 {
    unsafe {
        let vector: &[f32] = RawArray::from_array(vector).unwrap().data().as_ref();
        let other: &[f32] = RawArray::from_array(other).unwrap().data().as_ref();
        let len = vector.len() as i32;
        let dot = blas::sdot(len, vector, 1, other, 1);
        let a_norm = blas::snrm2(len, vector, 1);
        let b_norm = blas::snrm2(len, other, 1);
        dot / (a_norm * b_norm)
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "cosine_similarity")]
fn cosine_similarity_d(vector: Array<f64>, other: Array<f64>) -> f64 {
    unsafe {
        let vector: &[f64] = RawArray::from_array(vector).unwrap().data().as_ref();
        let other: &[f64] = RawArray::from_array(other).unwrap().data().as_ref();
        let len = vector.len() as i32;
        let dot = blas::ddot(len, vector, 1, other, 1);
        let a_norm = blas::dnrm2(len, vector, 1);
        let b_norm = blas::dnrm2(len, other, 1);
        dot / (a_norm * b_norm)
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct SumS;

#[pg_aggregate(parrallel_safe)]
impl Aggregate for SumS {
    const NAME: &'static str = "sum";
    type Args = Option<Vec<f32>>;
    type State = Option<Vec<f32>>;
    type Finalize = Vec<f32>;

    #[pgrx(immutable, parallel_safe)]
    fn state<'a>(mut current: Self::State, arg: Self::Args, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match arg {
            None => {}
            Some(arg) => match current {
                None => {
                    _ = current.insert(arg);
                }
                Some(ref mut vec) => {
                    for (i, v) in arg.iter().enumerate() {
                        vec[i] += v;
                    }
                }
            },
        }
        current
    }

    #[pgrx(immutable, parallel_safe)]
    fn combine(mut first: Self::State, second: Self::State, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match (&mut first, &second) {
            (None, None) => None,
            (Some(_), None) => first,
            (None, Some(_)) => second,
            (Some(first_inner), Some(second_inner)) => {
                for (i, v) in second_inner.iter().enumerate() {
                    first_inner[i] += v;
                }
                first
            }
        }
    }

    #[pgrx(immutable, parallel_safe)]
    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(Vec::new);

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct SumD;

#[pg_aggregate(parrallel_safe)]
impl Aggregate for SumD {
    const NAME: &'static str = "sum";
    type Args = Option<Vec<f64>>;
    type State = Option<Vec<f64>>;
    type Finalize = Vec<f64>;

    #[pgrx(immutable, parallel_safe)]
    fn state(mut current: Self::State, arg: Self::Args, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match arg {
            None => {}
            Some(arg) => match current {
                None => {
                    _ = current.insert(arg);
                }
                Some(ref mut vec) => {
                    for (i, v) in arg.iter().enumerate() {
                        vec[i] += v;
                    }
                }
            },
        }
        current
    }

    #[pgrx(immutable, parallel_safe)]
    fn combine(mut first: Self::State, second: Self::State, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match (&mut first, &second) {
            (None, None) => None,
            (Some(_), None) => first,
            (None, Some(_)) => second,
            (Some(first_inner), Some(second_inner)) => {
                for (i, v) in second_inner.iter().enumerate() {
                    first_inner[i] += v;
                }
                first
            }
        }
    }

    #[pgrx(immutable, parallel_safe)]
    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(Vec::new);

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MaxAbsS;

#[pg_aggregate(parrallel_safe)]
impl Aggregate for MaxAbsS {
    const NAME: &'static str = "max_abs";
    type Args = Option<Vec<f32>>;
    type State = Option<Vec<f32>>;
    type Finalize = Vec<f32>;

    #[pgrx(immutable, parallel_safe)]
    fn state(mut current: Self::State, arg: Self::Args, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match arg {
            None => {}
            Some(arg) => match current {
                None => {
                    _ = current.insert(arg.into_iter().map(|v| v.abs()).collect());
                }
                Some(ref mut vec) => {
                    for (i, &v) in arg.iter().enumerate() {
                        if v.abs() > vec[i].abs() {
                            vec[i] = v.abs();
                        }
                    }
                }
            },
        }
        current
    }

    #[pgrx(immutable, parallel_safe)]
    fn combine(mut first: Self::State, second: Self::State, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match (&mut first, &second) {
            (None, None) => None,
            (Some(_), None) => first,
            (None, Some(_)) => second,
            (Some(first_inner), Some(second_inner)) => {
                for (i, &v) in second_inner.iter().enumerate() {
                    if v.abs() > first_inner[i].abs() {
                        first_inner[i] = v.abs();
                    }
                }
                first
            }
        }
    }

    #[pgrx(immutable, parallel_safe)]
    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(Vec::new);

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MaxAbsD {}

#[pg_aggregate(parrallel_safe)]
impl Aggregate for MaxAbsD {
    const NAME: &'static str = "max_abs";
    type Args = Option<Vec<f64>>;
    type State = Option<Vec<f64>>;
    type Finalize = Vec<f64>;

    #[pgrx(immutable, parallel_safe)]
    fn state(mut current: Self::State, arg: Self::Args, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match arg {
            None => {}
            Some(arg) => match current {
                None => {
                    _ = current.insert(arg.into_iter().map(|v| v.abs()).collect());
                }
                Some(ref mut vec) => {
                    for (i, &v) in arg.iter().enumerate() {
                        if v.abs() > vec[i].abs() {
                            vec[i] = v.abs();
                        }
                    }
                }
            },
        }
        current
    }

    #[pgrx(immutable, parallel_safe)]
    fn combine(mut first: Self::State, second: Self::State, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match (&mut first, &second) {
            (None, None) => None,
            (Some(_), None) => first,
            (None, Some(_)) => second,
            (Some(first_inner), Some(second_inner)) => {
                for (i, &v) in second_inner.iter().enumerate() {
                    if v.abs() > first_inner[i].abs() {
                        first_inner[i] = v.abs();
                    }
                }
                first
            }
        }
    }

    #[pgrx(immutable, parallel_safe)]
    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(Vec::new);

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MaxS;

#[pg_aggregate(parrallel_safe)]
impl Aggregate for MaxS {
    const NAME: &'static str = "max";
    type Args = Option<Vec<f32>>;
    type State = Option<Vec<f32>>;
    type Finalize = Vec<f32>;

    #[pgrx(immutable, parallel_safe)]
    fn state(mut current: Self::State, arg: Self::Args, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match arg {
            None => {}
            Some(arg) => match current {
                None => {
                    _ = current.insert(arg);
                }
                Some(ref mut vec) => {
                    for (i, &v) in arg.iter().enumerate() {
                        if v > vec[i] {
                            vec[i] = v;
                        }
                    }
                }
            },
        }
        current
    }

    #[pgrx(immutable, parallel_safe)]
    fn combine(mut first: Self::State, second: Self::State, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match (&mut first, &second) {
            (None, None) => None,
            (Some(_), None) => first,
            (None, Some(_)) => second,
            (Some(first_inner), Some(second_inner)) => {
                for (i, &v) in second_inner.iter().enumerate() {
                    if v > first_inner[i] {
                        first_inner[i] = v;
                    }
                }
                first
            }
        }
    }

    #[pgrx(immutable, parallel_safe)]
    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(Vec::new);

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MaxD {}

#[pg_aggregate(parrallel_safe)]
impl Aggregate for MaxD {
    const NAME: &'static str = "max";
    type Args = Option<Vec<f64>>;
    type State = Option<Vec<f64>>;
    type Finalize = Vec<f64>;

    #[pgrx(immutable, parallel_safe)]
    fn state(mut current: Self::State, arg: Self::Args, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match arg {
            None => {}
            Some(arg) => match current {
                None => {
                    _ = current.insert(arg);
                }
                Some(ref mut vec) => {
                    for (i, &v) in arg.iter().enumerate() {
                        if v > vec[i] {
                            vec[i] = v;
                        }
                    }
                }
            },
        }
        current
    }

    #[pgrx(immutable, parallel_safe)]
    fn combine(mut first: Self::State, second: Self::State, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match (&mut first, &second) {
            (None, None) => None,
            (Some(_), None) => first,
            (None, Some(_)) => second,
            (Some(first_inner), Some(second_inner)) => {
                for (i, &v) in second_inner.iter().enumerate() {
                    if v > first_inner[i] {
                        first_inner[i] = v;
                    }
                }
                first
            }
        }
    }

    #[pgrx(immutable, parallel_safe)]
    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(Vec::new);

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MinS;

#[pg_aggregate(parrallel_safe)]
impl Aggregate for MinS {
    const NAME: &'static str = "min";
    type Args = Option<Vec<f32>>;
    type State = Option<Vec<f32>>;
    type Finalize = Vec<f32>;

    #[pgrx(immutable, parallel_safe)]
    fn state(mut current: Self::State, arg: Self::Args, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match arg {
            None => {}
            Some(arg) => match current {
                None => {
                    _ = current.insert(arg);
                }
                Some(ref mut vec) => {
                    for (i, &v) in arg.iter().enumerate() {
                        if v < vec[i] {
                            vec[i] = v;
                        }
                    }
                }
            },
        }
        current
    }

    #[pgrx(immutable, parallel_safe)]
    fn combine(mut first: Self::State, second: Self::State, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match (&mut first, &second) {
            (None, None) => None,
            (Some(_), None) => first,
            (None, Some(_)) => second,
            (Some(first_inner), Some(second_inner)) => {
                for (i, &v) in second_inner.iter().enumerate() {
                    if v < first_inner[i] {
                        first_inner[i] = v;
                    }
                }
                first
            }
        }
    }

    #[pgrx(immutable, parallel_safe)]
    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(Vec::new);

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MinD {}

#[pg_aggregate(parrallel_safe)]
impl Aggregate for MinD {
    const NAME: &'static str = "min";
    type Args = Option<Vec<f64>>;
    type State = Option<Vec<f64>>;
    type Finalize = Vec<f64>;

    #[pgrx(immutable, parallel_safe)]
    fn state(mut current: Self::State, arg: Self::Args, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match arg {
            None => {}
            Some(arg) => match current {
                None => {
                    _ = current.insert(arg);
                }
                Some(ref mut vec) => {
                    for (i, &v) in arg.iter().enumerate() {
                        if v < vec[i] {
                            vec[i] = v;
                        }
                    }
                }
            },
        }
        current
    }

    #[pgrx(immutable, parallel_safe)]
    fn combine(mut first: Self::State, second: Self::State, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match (&mut first, &second) {
            (None, None) => None,
            (Some(_), None) => first,
            (None, Some(_)) => second,
            (Some(first_inner), Some(second_inner)) => {
                for (i, &v) in second_inner.iter().enumerate() {
                    if v < first_inner[i] {
                        first_inner[i] = v;
                    }
                }
                first
            }
        }
    }

    #[pgrx(immutable, parallel_safe)]
    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(Vec::new);

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MinAbsS;

#[pg_aggregate(parrallel_safe)]
impl Aggregate for MinAbsS {
    const NAME: &'static str = "min_abs";
    type Args = Option<Vec<f32>>;
    type State = Option<Vec<f32>>;
    type Finalize = Vec<f32>;

    #[pgrx(immutable, parallel_safe)]
    fn state(mut current: Self::State, arg: Self::Args, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match arg {
            None => {}
            Some(arg) => match current {
                None => {
                    _ = current.insert(arg.into_iter().map(|v| v.abs()).collect());
                }
                Some(ref mut vec) => {
                    for (i, &v) in arg.iter().enumerate() {
                        if v.abs() < vec[i].abs() {
                            vec[i] = v.abs();
                        }
                    }
                }
            },
        }
        current
    }

    #[pgrx(immutable, parallel_safe)]
    fn combine(mut first: Self::State, second: Self::State, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match (&mut first, &second) {
            (None, None) => None,
            (Some(_), None) => first,
            (None, Some(_)) => second,
            (Some(first_inner), Some(second_inner)) => {
                for (i, &v) in second_inner.iter().enumerate() {
                    if v.abs() < first_inner[i].abs() {
                        first_inner[i] = v.abs();
                    }
                }
                first
            }
        }
    }

    #[pgrx(immutable, parallel_safe)]
    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(Vec::new);

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MinAbsD {}

#[pg_aggregate(parrallel_safe)]
impl Aggregate for MinAbsD {
    const NAME: &'static str = "min_abs";
    type Args = Option<Vec<f64>>;
    type State = Option<Vec<f64>>;
    type Finalize = Vec<f64>;

    #[pgrx(immutable, parallel_safe)]
    fn state(mut current: Self::State, arg: Self::Args, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match arg {
            None => {}
            Some(arg) => match current {
                None => {
                    _ = current.insert(arg.into_iter().map(|v| v.abs()).collect());
                }
                Some(ref mut vec) => {
                    for (i, &v) in arg.iter().enumerate() {
                        if v.abs() < vec[i].abs() {
                            vec[i] = v.abs();
                        }
                    }
                }
            },
        }
        current
    }

    #[pgrx(immutable, parallel_safe)]
    fn combine(mut first: Self::State, second: Self::State, _fcinfo: pg_sys::FunctionCallInfo) -> Self::State {
        match (&mut first, &second) {
            (None, None) => None,
            (Some(_), None) => first,
            (None, Some(_)) => second,
            (Some(first_inner), Some(second_inner)) => {
                for (i, &v) in second_inner.iter().enumerate() {
                    if v.abs() < first_inner[i].abs() {
                        first_inner[i] = v.abs();
                    }
                }
                first
            }
        }
    }

    #[pgrx(immutable, parallel_safe)]
    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(Vec::new);

        inner.clone()
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    const F32_TOLERANCE: f32 = 3e-7;
    const F64_TOLERANCE: f64 = 5e-16;

    #[pg_test]
    fn test_add_scalar_s() {
        let result = Spi::get_one::<Vec<f32>>("SELECT pgml.add(ARRAY[1,2,3]::float4[], 1)");
        assert_eq!(result, Ok(Some([2.0, 3.0, 4.0].to_vec())));
    }

    #[pg_test]
    fn test_add_scalar_d() {
        let result = Spi::get_one::<Vec<f64>>("SELECT pgml.add(ARRAY[1,2,3]::float8[], 1)");
        assert_eq!(result, Ok(Some([2.0, 3.0, 4.0].to_vec())));
    }

    #[pg_test]
    fn test_subtract_scalar_s() {
        let result = Spi::get_one::<Vec<f32>>("SELECT pgml.subtract(ARRAY[1,2,3]::float4[], 1)");
        assert_eq!(result, Ok(Some([0.0, 1.0, 2.0].to_vec())));
    }

    #[pg_test]
    fn test_subtract_scalar_d() {
        let result = Spi::get_one::<Vec<f64>>("SELECT pgml.subtract(ARRAY[1,2,3]::float8[], 1)");
        assert_eq!(result, Ok(Some([0.0, 1.0, 2.0].to_vec())));
    }

    #[pg_test]
    fn test_multiply_scalar_s() {
        let result = Spi::get_one::<Vec<f32>>("SELECT pgml.multiply(ARRAY[1,2,3]::float4[], 2)");
        assert_eq!(result, Ok(Some([2.0, 4.0, 6.0].to_vec())));
    }

    #[pg_test]
    fn test_multiply_scalar_d() {
        let result = Spi::get_one::<Vec<f64>>("SELECT pgml.multiply(ARRAY[1,2,3]::float8[], 2)");
        assert_eq!(result, Ok(Some([2.0, 4.0, 6.0].to_vec())));
    }

    #[pg_test]
    fn test_divide_scalar_s() {
        let result = Spi::get_one::<Vec<f32>>("SELECT pgml.divide(ARRAY[1,2,3]::float4[], 10)");
        assert_eq!(result, Ok(Some([0.1, 0.2, 0.3].to_vec())));
    }

    #[pg_test]
    fn test_divide_scalar_d() {
        let result = Spi::get_one::<Vec<f64>>("SELECT pgml.divide(ARRAY[1,2,3]::float8[], 10)");
        assert_eq!(result, Ok(Some([0.1, 0.2, 0.3].to_vec())));
    }

    #[pg_test]
    fn test_add_vector_s() {
        let result =
            Spi::get_one::<Vec<f32>>("SELECT pgml.add(ARRAY[1,2,3]::float4[], ARRAY[1.0, 2.0, 3.0]::float4[])");
        assert_eq!(result, Ok(Some([2.0, 4.0, 6.0].to_vec())));
    }

    #[pg_test]
    fn test_add_vector_d() {
        let result =
            Spi::get_one::<Vec<f64>>("SELECT pgml.add(ARRAY[1,2,3]::float8[], ARRAY[1.0, 2.0, 3.0]::float8[])");
        assert_eq!(result, Ok(Some([2.0, 4.0, 6.0].to_vec())));
    }

    #[pg_test]
    fn test_subtract_vector_s() {
        let result =
            Spi::get_one::<Vec<f32>>("SELECT pgml.subtract(ARRAY[1,2,3]::float4[], ARRAY[1.0, 2.0, 3.0]::float4[])");
        assert_eq!(result, Ok(Some([0.0, 0.0, 0.0].to_vec())));
    }

    #[pg_test]
    fn test_subtract_vector_d() {
        let result =
            Spi::get_one::<Vec<f64>>("SELECT pgml.subtract(ARRAY[1,2,3]::float8[], ARRAY[1.0, 2.0, 3.0]::float8[])");
        assert_eq!(result, Ok(Some([0.0, 0.0, 0.0].to_vec())));
    }

    #[pg_test]
    fn test_multiply_vector_s() {
        let result =
            Spi::get_one::<Vec<f32>>("SELECT pgml.subtract(ARRAY[1,2,3]::float4[], ARRAY[1.0, 2.0, 3.0]::float4[])");
        assert_eq!(result, Ok(Some([0.0, 0.0, 0.0].to_vec())));
    }

    #[pg_test]
    fn test_multiply_vector_d() {
        let result =
            Spi::get_one::<Vec<f64>>("SELECT pgml.multiply(ARRAY[1,2,3]::float8[], ARRAY[1.0, 2.0, 3.0]::float8[])");
        assert_eq!(result, Ok(Some([1.0, 4.0, 9.0].to_vec())));
    }

    #[pg_test]
    fn test_divide_vector_s() {
        let result =
            Spi::get_one::<Vec<f32>>("SELECT pgml.divide(ARRAY[1,2,3]::float4[], ARRAY[1.0, 2.0, 3.0]::float4[])");
        assert_eq!(result, Ok(Some([1.0, 1.0, 1.0].to_vec())));
    }

    #[pg_test]
    fn test_divide_vector_d() {
        let result =
            Spi::get_one::<Vec<f64>>("SELECT pgml.divide(ARRAY[1,2,3]::float8[], ARRAY[1.0, 2.0, 3.0]::float8[])");
        assert_eq!(result, Ok(Some([1.0, 1.0, 1.0].to_vec())));
    }

    #[pg_test]
    fn test_norm_l0_s() {
        let result = Spi::get_one::<f32>("SELECT pgml.norm_l0(ARRAY[1,2,3]::float4[])");
        assert_eq!(result, Ok(Some(3.0)));
    }

    #[pg_test]
    fn test_norm_l0_d() {
        let result = Spi::get_one::<f64>("SELECT pgml.norm_l0(ARRAY[1,2,3]::float8[])");
        assert_eq!(result, Ok(Some(3.0)));
    }

    #[pg_test]
    fn test_norm_l1_s() {
        let result = Spi::get_one::<f32>("SELECT pgml.norm_l1(ARRAY[1,2,3]::float4[])");
        assert_eq!(result, Ok(Some(6.0)));
    }

    #[pg_test]
    fn test_norm_l1_d() {
        let result = Spi::get_one::<f64>("SELECT pgml.norm_l1(ARRAY[1,2,3]::float8[])");
        assert_eq!(result, Ok(Some(6.0)));
    }

    #[pg_test]
    fn test_norm_l2_s() {
        let got = Spi::get_one::<f32>("SELECT pgml.norm_l2(ARRAY[1,2,3]::float4[])")
            .unwrap()
            .unwrap();
        let want = 3.7416575;
        let diff = (got - want).abs();
        assert!(diff < F32_TOLERANCE);
    }

    #[pg_test]
    fn test_norm_l2_d() {
        let got = Spi::get_one::<f64>("SELECT pgml.norm_l2(ARRAY[1,2,3]::float8[])")
            .unwrap()
            .unwrap();
        let want = 3.7416573867739413;
        let diff = (got - want).abs();
        assert!(diff < F64_TOLERANCE);
    }

    #[pg_test]
    fn test_norm_max_s() {
        let result = Spi::get_one::<f32>("SELECT pgml.norm_max(ARRAY[1,2,3]::float4[])");
        assert_eq!(result, Ok(Some(3.0)));

        let result = Spi::get_one::<f32>("SELECT pgml.norm_max(ARRAY[1,2,3,-4]::float4[])");
        assert_eq!(result, Ok(Some(4.0)));
    }

    #[pg_test]
    fn test_norm_max_d() {
        let result = Spi::get_one::<f64>("SELECT pgml.norm_max(ARRAY[1,2,3]::float8[])");
        assert_eq!(result, Ok(Some(3.0)));

        let result = Spi::get_one::<f64>("SELECT pgml.norm_max(ARRAY[1,2,3,-4]::float8[])");
        assert_eq!(result, Ok(Some(4.0)));
    }

    #[pg_test]
    fn test_normalize_l1_s() {
        let result = Spi::get_one::<Vec<f32>>("SELECT pgml.normalize_l1(ARRAY[1,2,3]::float4[])");
        assert_eq!(result, Ok(Some([0.16666667, 0.33333334, 0.5].to_vec())));
    }

    #[pg_test]
    fn test_normalize_l1_d() {
        let result = Spi::get_one::<Vec<f64>>("SELECT pgml.normalize_l1(ARRAY[1,2,3]::float8[])");
        assert_eq!(
            result,
            Ok(Some([0.16666666666666666, 0.3333333333333333, 0.5].to_vec()))
        );
    }

    #[pg_test]
    fn test_normalize_l2_s() {
        let got = Spi::get_one::<Vec<f32>>("SELECT pgml.normalize_l2(ARRAY[1,2,3]::float4[])")
            .unwrap()
            .unwrap();
        let want = [0.26726124, 0.5345225, 0.8017837];
        assert!(got
            .iter()
            .zip(want)
            .all(|(got, want)| (got - want).abs() < F32_TOLERANCE));
    }

    #[pg_test]
    fn test_normalize_l2_d() {
        let got = Spi::get_one::<Vec<f64>>("SELECT pgml.normalize_l2(ARRAY[1,2,3]::float8[])")
            .unwrap()
            .unwrap();
        let want = [0.2672612419124244, 0.5345224838248488, 0.8017837257372732].to_vec();
        assert!(got
            .iter()
            .zip(want)
            .all(|(got, want)| (got - want).abs() < F64_TOLERANCE));
    }

    #[pg_test]
    fn test_normalize_max_s() {
        let result = Spi::get_one::<Vec<f32>>("SELECT pgml.normalize_max(ARRAY[1,2,3]::float4[])");
        assert_eq!(result, Ok(Some([0.33333334, 0.6666667, 1.0].to_vec())));
    }

    #[pg_test]
    fn test_normalize_max_d() {
        let result = Spi::get_one::<Vec<f64>>("SELECT pgml.normalize_max(ARRAY[1,2,3]::float8[])");
        assert_eq!(result, Ok(Some([0.3333333333333333, 0.6666666666666666, 1.0].to_vec())));
    }

    #[pg_test]
    fn test_distance_l1_s() {
        let result = Spi::get_one::<f32>("SELECT pgml.distance_l1(ARRAY[1,2,3]::float4[],ARRAY[1,2,3]::float4[])");
        assert_eq!(result, Ok(Some(0.0)));
    }

    #[pg_test]
    fn test_distance_l1_d() {
        let result = Spi::get_one::<f64>("SELECT pgml.distance_l1(ARRAY[1,2,3]::float8[],ARRAY[1,2,3]::float8[])");
        assert_eq!(result, Ok(Some(0.0)));
    }

    #[pg_test]
    fn test_distance_l2_s() {
        let result = Spi::get_one::<f32>("SELECT pgml.distance_l2(ARRAY[1,2,3]::float4[],ARRAY[1,2,3]::float4[])");
        assert_eq!(result, Ok(Some(0.0)));
    }

    #[pg_test]
    fn test_distance_l2_d() {
        let result = Spi::get_one::<f64>("SELECT pgml.distance_l2(ARRAY[1,2,3]::float8[],ARRAY[1,2,3]::float8[])");
        assert_eq!(result, Ok(Some(0.0)));
    }

    #[pg_test]
    fn test_dot_product_s() {
        let result = Spi::get_one::<f32>("SELECT pgml.dot_product(ARRAY[1,2,3]::float4[],ARRAY[1,2,3]::float4[])");
        assert_eq!(result, Ok(Some(14.0)));

        let result = Spi::get_one::<f32>("SELECT pgml.dot_product(ARRAY[1,2,3]::float4[],ARRAY[2,3,4]::float4[])");
        assert_eq!(result, Ok(Some(20.0)));
    }

    #[pg_test]
    fn test_dot_product_d() {
        let result = Spi::get_one::<f64>("SELECT pgml.dot_product(ARRAY[1,2,3]::float8[],ARRAY[1,2,3]::float8[])");
        assert_eq!(result, Ok(Some(14.0)));

        let result = Spi::get_one::<f64>("SELECT pgml.dot_product(ARRAY[1,2,3]::float8[],ARRAY[2,3,4]::float8[])");
        assert_eq!(result, Ok(Some(20.0)));
    }

    #[pg_test]
    fn test_cosine_similarity_s() {
        let got = Spi::get_one::<f32>(
            "SELECT pgml.cosine_similarity(ARRAY[1,2,3]::float4[], ARRAY[1.0, 2.0, 3.0]::float4[])",
        )
        .unwrap()
        .unwrap();
        let want = 1.0;
        assert!((got - want).abs() < F32_TOLERANCE);

        let got = Spi::get_one::<f32>(
            "SELECT pgml.cosine_similarity(ARRAY[1,2,3]::float4[], ARRAY[2.0, 3.0, 4.0]::float4[])",
        )
        .unwrap()
        .unwrap();
        let want = 0.9925833;
        assert!((got - want).abs() < F32_TOLERANCE);

        let got = Spi::get_one::<f32>(
            "SELECT pgml.cosine_similarity(ARRAY[1,1,1,1,1,0,0]::float4[], ARRAY[0,0,1,1,0,1,1]::float4[])",
        )
        .unwrap()
        .unwrap();
        let want = 0.4472136;
        assert!((got - want).abs() < F32_TOLERANCE);
    }

    #[pg_test]
    fn test_cosine_similarity_d() {
        let got = Spi::get_one::<f64>(
            "SELECT pgml.cosine_similarity(ARRAY[1,2,3]::float8[], ARRAY[1.0, 2.0, 3.0]::float8[])",
        )
        .unwrap()
        .unwrap();
        let want = 1.0;
        assert!((got - want).abs() < F64_TOLERANCE);

        let got = Spi::get_one::<f64>(
            "SELECT pgml.cosine_similarity(ARRAY[1,2,3]::float8[], ARRAY[2.0, 3.0, 4.0]::float8[])",
        )
        .unwrap()
        .unwrap();
        let want = 0.9925833339709303;
        assert!((got - want).abs() < F64_TOLERANCE);

        let got = Spi::get_one::<f64>(
            "SELECT pgml.cosine_similarity(ARRAY[1,1,1,1,1,0,0]::float8[], ARRAY[0,0,1,1,0,1,1]::float8[])",
        )
        .unwrap()
        .unwrap();
        let want = 0.4472135954999579;
        assert!((got - want).abs() < F64_TOLERANCE);
    }
}
