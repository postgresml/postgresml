use pgrx::*;

#[pg_extern(immutable, parallel_safe, strict, name = "add")]
fn add_scalar_s(vector: Vec<f32>, addend: f32) -> Vec<f32> {
    vector.as_slice().iter().map(|a| a + addend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "add")]
fn add_scalar_d(vector: Vec<f64>, addend: f64) -> Vec<f64> {
    vector.as_slice().iter().map(|a| a + addend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "subtract")]
fn subtract_scalar_s(vector: Vec<f32>, subtahend: f32) -> Vec<f32> {
    vector.as_slice().iter().map(|a| a - subtahend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "subtract")]
fn subtract_scalar_d(vector: Vec<f64>, subtahend: f64) -> Vec<f64> {
    vector.as_slice().iter().map(|a| a - subtahend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "multiply")]
fn multiply_scalar_s(vector: Vec<f32>, multiplicand: f32) -> Vec<f32> {
    vector.as_slice().iter().map(|a| a * multiplicand).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "multiply")]
fn multiply_scalar_d(vector: Vec<f64>, multiplicand: f64) -> Vec<f64> {
    vector.as_slice().iter().map(|a| a * multiplicand).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "divide")]
fn divide_scalar_s(vector: Vec<f32>, dividend: f32) -> Vec<f32> {
    vector.as_slice().iter().map(|a| a / dividend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "divide")]
fn divide_scalar_d(vector: Vec<f64>, dividend: f64) -> Vec<f64> {
    vector.as_slice().iter().map(|a| a / dividend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "add")]
fn add_vector_s(vector: Vec<f32>, addend: Vec<f32>) -> Vec<f32> {
    vector
        .as_slice()
        .iter()
        .zip(addend.as_slice().iter())
        .map(|(a, b)| a + b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "add")]
fn add_vector_d(vector: Vec<f64>, addend: Vec<f64>) -> Vec<f64> {
    vector
        .as_slice()
        .iter()
        .zip(addend.as_slice().iter())
        .map(|(a, b)| a + b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "subtract")]
fn subtract_vector_s(vector: Vec<f32>, subtahend: Vec<f32>) -> Vec<f32> {
    vector
        .as_slice()
        .iter()
        .zip(subtahend.as_slice().iter())
        .map(|(a, b)| a - b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "subtract")]
fn subtract_vector_d(vector: Vec<f64>, subtahend: Vec<f64>) -> Vec<f64> {
    vector
        .as_slice()
        .iter()
        .zip(subtahend.as_slice().iter())
        .map(|(a, b)| a - b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "multiply")]
fn multiply_vector_s(vector: Vec<f32>, multiplicand: Vec<f32>) -> Vec<f32> {
    vector
        .as_slice()
        .iter()
        .zip(multiplicand.as_slice().iter())
        .map(|(a, b)| a * b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "multiply")]
fn multiply_vector_d(vector: Vec<f64>, multiplicand: Vec<f64>) -> Vec<f64> {
    vector
        .as_slice()
        .iter()
        .zip(multiplicand.as_slice().iter())
        .map(|(a, b)| a * b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "divide")]
fn divide_vector_s(vector: Vec<f32>, dividend: Vec<f32>) -> Vec<f32> {
    vector
        .as_slice()
        .iter()
        .zip(dividend.as_slice().iter())
        .map(|(a, b)| a / b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "divide")]
fn divide_vector_d(vector: Vec<f64>, dividend: Vec<f64>) -> Vec<f64> {
    vector
        .as_slice()
        .iter()
        .zip(dividend.as_slice().iter())
        .map(|(a, b)| a / b)
        .collect()
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_l0")]
fn norm_l0_s(vector: Vec<f32>) -> f32 {
    vector
        .as_slice()
        .iter()
        .map(|a| if *a == 0.0 { 0.0 } else { 1.0 })
        .sum()
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_l0")]
fn norm_l0_d(vector: Vec<f64>) -> f64 {
    vector
        .as_slice()
        .iter()
        .map(|a| if *a == 0.0 { 0.0 } else { 1.0 })
        .sum()
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_l1")]
fn norm_l1_s(vector: Vec<f32>) -> f32 {
    unsafe { blas::sasum(vector.len().try_into().unwrap(), vector.as_slice(), 1) }
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_l1")]
fn norm_l1_d(vector: Vec<f64>) -> f64 {
    unsafe { blas::dasum(vector.len().try_into().unwrap(), vector.as_slice(), 1) }
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_l2")]
fn norm_l2_s(vector: Vec<f32>) -> f32 {
    unsafe { blas::snrm2(vector.len().try_into().unwrap(), vector.as_slice(), 1) }
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_l2")]
fn norm_l2_d(vector: Vec<f64>) -> f64 {
    unsafe { blas::dnrm2(vector.len().try_into().unwrap(), vector.as_slice(), 1) }
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_max")]
fn norm_max_s(vector: Vec<f32>) -> f32 {
    unsafe {
        let index = blas::isamax(vector.len().try_into().unwrap(), vector.as_slice(), 1);
        vector[index - 1].abs()
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "norm_max")]
fn norm_max_d(vector: Vec<f64>) -> f64 {
    unsafe {
        let index = blas::idamax(vector.len().try_into().unwrap(), vector.as_slice(), 1);
        vector[index - 1].abs()
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "normalize_l1")]
fn normalize_l1_s(vector: Vec<f32>) -> Vec<f32> {
    let norm: f32;
    unsafe {
        norm = blas::sasum(vector.len().try_into().unwrap(), vector.as_slice(), 1);
    }
    divide_scalar_s(vector, norm)
}

#[pg_extern(immutable, parallel_safe, strict, name = "normalize_l1")]
fn normalize_l1_d(vector: Vec<f64>) -> Vec<f64> {
    let norm: f64;
    unsafe {
        norm = blas::dasum(vector.len().try_into().unwrap(), vector.as_slice(), 1);
    }
    divide_scalar_d(vector, norm)
}

#[pg_extern(immutable, parallel_safe, strict, name = "normalize_l2")]
fn normalize_l2_s(vector: Vec<f32>) -> Vec<f32> {
    let norm: f32;
    unsafe {
        norm = blas::snrm2(vector.len().try_into().unwrap(), vector.as_slice(), 1);
    }
    divide_scalar_s(vector, norm)
}

#[pg_extern(immutable, parallel_safe, strict, name = "normalize_l2")]
fn normalize_l2_d(vector: Vec<f64>) -> Vec<f64> {
    let norm: f64;
    unsafe {
        norm = blas::dnrm2(vector.len().try_into().unwrap(), vector.as_slice(), 1);
    }
    divide_scalar_d(vector, norm)
}

#[pg_extern(immutable, parallel_safe, strict, name = "normalize_max")]
fn normalize_max_s(vector: Vec<f32>) -> Vec<f32> {
    let norm;
    unsafe {
        let index = blas::isamax(vector.len().try_into().unwrap(), vector.as_slice(), 1);
        norm = vector[index - 1].abs();
    }
    divide_scalar_s(vector, norm)
}

#[pg_extern(immutable, parallel_safe, strict, name = "normalize_max")]
fn normalize_max_d(vector: Vec<f64>) -> Vec<f64> {
    let norm;
    unsafe {
        let index = blas::idamax(vector.len().try_into().unwrap(), vector.as_slice(), 1);
        norm = vector[index - 1].abs();
    }
    divide_scalar_d(vector, norm)
}

#[pg_extern(immutable, parallel_safe, strict, name = "distance_l1")]
fn distance_l1_s(vector: Vec<f32>, other: Vec<f32>) -> f32 {
    vector
        .as_slice()
        .iter()
        .zip(other.as_slice().iter())
        .map(|(a, b)| (a - b).abs())
        .sum()
}

#[pg_extern(immutable, parallel_safe, strict, name = "distance_l1")]
fn distance_l1_d(vector: Vec<f64>, other: Vec<f64>) -> f64 {
    vector
        .as_slice()
        .iter()
        .zip(other.as_slice().iter())
        .map(|(a, b)| (a - b).abs())
        .sum()
}

#[pg_extern(immutable, parallel_safe, strict, name = "distance_l2")]
fn distance_l2_s(vector: Vec<f32>, other: Vec<f32>) -> f32 {
    vector
        .as_slice()
        .iter()
        .zip(other.as_slice().iter())
        .map(|(a, b)| (a - b).powf(2.0))
        .sum::<f32>()
        .sqrt()
}

#[pg_extern(immutable, parallel_safe, strict, name = "distance_l2")]
fn distance_l2_d(vector: Vec<f64>, other: Vec<f64>) -> f64 {
    vector
        .as_slice()
        .iter()
        .zip(other.as_slice().iter())
        .map(|(a, b)| (a - b).powf(2.0))
        .sum::<f64>()
        .sqrt()
}

#[pg_extern(immutable, parallel_safe, strict, name = "dot_product")]
fn dot_product_s(vector: Vec<f32>, other: Vec<f32>) -> f32 {
    unsafe {
        blas::sdot(
            vector.len().try_into().unwrap(),
            vector.as_slice(),
            1,
            other.as_slice(),
            1,
        )
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "dot_product")]
fn dot_product_d(vector: Vec<f64>, other: Vec<f64>) -> f64 {
    unsafe {
        blas::ddot(
            vector.len().try_into().unwrap(),
            vector.as_slice(),
            1,
            other.as_slice(),
            1,
        )
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "cosine_similarity")]
fn cosine_similarity_s(vector: Vec<f32>, other: Vec<f32>) -> f32 {
    unsafe {
        let dot = blas::sdot(
            vector.len().try_into().unwrap(),
            vector.as_slice(),
            1,
            other.as_slice(),
            1,
        );
        let a_norm = blas::snrm2(vector.len().try_into().unwrap(), vector.as_slice(), 1);
        let b_norm = blas::snrm2(other.len().try_into().unwrap(), other.as_slice(), 1);
        dot / (a_norm * b_norm)
    }
}

#[pg_extern(immutable, parallel_safe, strict, name = "cosine_similarity")]
fn cosine_similarity_d(vector: Vec<f64>, other: Vec<f64>) -> f64 {
    unsafe {
        let dot = blas::ddot(
            vector.len().try_into().unwrap(),
            vector.as_slice(),
            1,
            other.as_slice(),
            1,
        );
        let a_norm = blas::dnrm2(vector.len().try_into().unwrap(), vector.as_slice(), 1);
        let b_norm = blas::dnrm2(other.len().try_into().unwrap(), other.as_slice(), 1);
        dot / (a_norm * b_norm)
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct SumS;

#[pg_aggregate]
impl Aggregate for SumS {
    const NAME: &'static str = "sum";
    type Args = Option<Vec<f32>>;
    type State = Option<Vec<f32>>;
    type Finalize = Vec<f32>;

    fn state(
        mut current: Self::State,
        arg: Self::Args,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn combine(
        mut first: Self::State,
        second: Self::State,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(|| Vec::new());

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct SumD;

#[pg_aggregate]
impl Aggregate for SumD {
    const NAME: &'static str = "sum";
    type Args = Option<Vec<f64>>;
    type State = Option<Vec<f64>>;
    type Finalize = Vec<f64>;

    fn state(
        mut current: Self::State,
        arg: Self::Args,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn combine(
        mut first: Self::State,
        second: Self::State,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(|| Vec::new());

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MaxAbsS;

#[pg_aggregate]
impl Aggregate for MaxAbsS {
    const NAME: &'static str = "max_abs";
    type Args = Option<Vec<f32>>;
    type State = Option<Vec<f32>>;
    type Finalize = Vec<f32>;

    fn state(
        mut current: Self::State,
        arg: Self::Args,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn combine(
        mut first: Self::State,
        second: Self::State,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(|| Vec::new());

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MaxAbsD {}

#[pg_aggregate]
impl Aggregate for MaxAbsD {
    const NAME: &'static str = "max_abs";
    type Args = Option<Vec<f64>>;
    type State = Option<Vec<f64>>;
    type Finalize = Vec<f64>;

    fn state(
        mut current: Self::State,
        arg: Self::Args,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn combine(
        mut first: Self::State,
        second: Self::State,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(|| Vec::new());

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MaxS;

#[pg_aggregate]
impl Aggregate for MaxS {
    const NAME: &'static str = "max";
    type Args = Option<Vec<f32>>;
    type State = Option<Vec<f32>>;
    type Finalize = Vec<f32>;

    fn state(
        mut current: Self::State,
        arg: Self::Args,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn combine(
        mut first: Self::State,
        second: Self::State,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(|| Vec::new());

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MaxD {}

#[pg_aggregate]
impl Aggregate for MaxD {
    const NAME: &'static str = "max";
    type Args = Option<Vec<f64>>;
    type State = Option<Vec<f64>>;
    type Finalize = Vec<f64>;

    fn state(
        mut current: Self::State,
        arg: Self::Args,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn combine(
        mut first: Self::State,
        second: Self::State,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(|| Vec::new());

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MinS;

#[pg_aggregate]
impl Aggregate for MinS {
    const NAME: &'static str = "min";
    type Args = Option<Vec<f32>>;
    type State = Option<Vec<f32>>;
    type Finalize = Vec<f32>;

    fn state(
        mut current: Self::State,
        arg: Self::Args,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn combine(
        mut first: Self::State,
        second: Self::State,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(|| Vec::new());

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MinD {}

#[pg_aggregate]
impl Aggregate for MinD {
    const NAME: &'static str = "min";
    type Args = Option<Vec<f64>>;
    type State = Option<Vec<f64>>;
    type Finalize = Vec<f64>;

    fn state(
        mut current: Self::State,
        arg: Self::Args,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn combine(
        mut first: Self::State,
        second: Self::State,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(|| Vec::new());

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MinAbsS;

#[pg_aggregate]
impl Aggregate for MinAbsS {
    const NAME: &'static str = "min_abs";
    type Args = Option<Vec<f32>>;
    type State = Option<Vec<f32>>;
    type Finalize = Vec<f32>;

    fn state(
        mut current: Self::State,
        arg: Self::Args,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn combine(
        mut first: Self::State,
        second: Self::State,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(|| Vec::new());

        inner.clone()
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct MinAbsD {}

#[pg_aggregate]
impl Aggregate for MinAbsD {
    const NAME: &'static str = "min_abs";
    type Args = Option<Vec<f64>>;
    type State = Option<Vec<f64>>;
    type Finalize = Vec<f64>;

    fn state(
        mut current: Self::State,
        arg: Self::Args,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn combine(
        mut first: Self::State,
        second: Self::State,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
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

    fn finalize(
        mut current: Self::State,
        _direct_arg: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let inner = current.get_or_insert_with(|| Vec::new());

        inner.clone()
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn test_add_scalar_s() {
        assert_eq!(
            add_scalar_s([1.0, 2.0, 3.0].to_vec(), 1.0),
            [2.0, 3.0, 4.0].to_vec()
        )
    }

    #[pg_test]
    fn test_add_scalar_d() {
        assert_eq!(
            add_scalar_d([1.0, 2.0, 3.0].to_vec(), 1.0),
            [2.0, 3.0, 4.0].to_vec()
        )
    }

    #[pg_test]
    fn test_subtract_scalar_s() {
        assert_eq!(
            subtract_scalar_s([1.0, 2.0, 3.0].to_vec(), 1.0),
            [0.0, 1.0, 2.0].to_vec()
        )
    }

    #[pg_test]
    fn test_subtract_scalar_d() {
        assert_eq!(
            subtract_scalar_d([1.0, 2.0, 3.0].to_vec(), 1.0),
            [0.0, 1.0, 2.0].to_vec()
        )
    }

    #[pg_test]
    fn test_multiply_scalar_s() {
        assert_eq!(
            multiply_scalar_d([1.0, 2.0, 3.0].to_vec(), 2.0),
            [2.0, 4.0, 6.0].to_vec()
        )
    }

    #[pg_test]
    fn test_multiply_scalar_d() {
        assert_eq!(
            multiply_scalar_d([1.0, 2.0, 3.0].to_vec(), 2.0),
            [2.0, 4.0, 6.0].to_vec()
        )
    }

    #[pg_test]
    fn test_divide_scalar_s() {
        assert_eq!(
            divide_scalar_s([2.0, 4.0, 6.0].to_vec(), 2.0),
            [1.0, 2.0, 3.0].to_vec()
        )
    }

    #[pg_test]
    fn test_divide_scalar_d() {
        assert_eq!(
            divide_scalar_d([2.0, 4.0, 6.0].to_vec(), 2.0),
            [1.0, 2.0, 3.0].to_vec()
        )
    }

    #[pg_test]
    fn test_add_vector_s() {
        assert_eq!(
            add_vector_s([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            [2.0, 4.0, 6.0].to_vec()
        )
    }

    #[pg_test]
    fn test_add_vector_d() {
        assert_eq!(
            add_vector_d([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            [2.0, 4.0, 6.0].to_vec()
        )
    }

    #[pg_test]
    fn test_subtract_vector_s() {
        assert_eq!(
            subtract_vector_s([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            [0.0, 0.0, 0.0].to_vec()
        )
    }

    #[pg_test]
    fn test_subtract_vector_d() {
        assert_eq!(
            subtract_vector_d([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            [0.0, 0.0, 0.0].to_vec()
        )
    }

    #[pg_test]
    fn test_multiply_vector_s() {
        assert_eq!(
            multiply_vector_s([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            [1.0, 4.0, 9.0].to_vec()
        )
    }

    #[pg_test]
    fn test_multiply_vector_d() {
        assert_eq!(
            multiply_vector_d([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            [1.0, 4.0, 9.0].to_vec()
        )
    }

    #[pg_test]
    fn test_divide_vector_s() {
        assert_eq!(
            divide_vector_s([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            [1.0, 1.0, 1.0].to_vec()
        )
    }

    #[pg_test]
    fn test_divide_vector_d() {
        assert_eq!(
            divide_vector_d([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            [1.0, 1.0, 1.0].to_vec()
        )
    }

    #[pg_test]
    fn test_norm_l0_s() {
        assert_eq!(norm_l0_s([1.0, 2.0, 3.0].to_vec()), 3.0)
    }

    #[pg_test]
    fn test_norm_l0_d() {
        assert_eq!(norm_l0_d([1.0, 2.0, 3.0].to_vec()), 3.0)
    }

    #[pg_test]
    fn test_norm_l1_s() {
        assert_eq!(norm_l1_s([1.0, 2.0, 3.0].to_vec()), 6.0)
    }

    #[pg_test]
    fn test_norm_l1_d() {
        assert_eq!(norm_l1_d([1.0, 2.0, 3.0].to_vec()), 6.0)
    }

    #[pg_test]
    fn test_norm_l2_s() {
        assert_eq!(norm_l2_s([1.0, 2.0, 3.0].to_vec()), 3.7416575);
    }

    #[pg_test]
    fn test_norm_l2_d() {
        assert_eq!(norm_l2_d([1.0, 2.0, 3.0].to_vec()), 3.7416573867739413);
    }

    #[pg_test]
    fn test_norm_max_s() {
        assert_eq!(norm_max_s([1.0, 2.0, 3.0].to_vec()), 3.0);
        assert_eq!(norm_max_s([1.0, 2.0, 3.0, -4.0].to_vec()), 4.0);
    }

    #[pg_test]
    fn test_norm_max_d() {
        assert_eq!(norm_max_d([1.0, 2.0, 3.0].to_vec()), 3.0);
        assert_eq!(norm_max_d([1.0, 2.0, 3.0, -4.0].to_vec()), 4.0);
    }

    #[pg_test]
    fn test_normalize_l1_s() {
        assert_eq!(
            normalize_l1_s([1.0, 2.0, 3.0].to_vec()),
            [0.16666667, 0.33333334, 0.5].to_vec()
        );
    }

    #[pg_test]
    fn test_normalize_l1_d() {
        assert_eq!(
            normalize_l1_d([1.0, 2.0, 3.0].to_vec()),
            [0.16666666666666666, 0.3333333333333333, 0.5].to_vec()
        );
    }

    #[pg_test]
    fn test_normalize_l2_s() {
        assert_eq!(
            normalize_l2_s([1.0, 2.0, 3.0].to_vec()),
            [0.26726124, 0.5345225, 0.8017837].to_vec()
        );
    }

    #[pg_test]
    fn test_normalize_l2_d() {
        assert_eq!(
            normalize_l2_d([1.0, 2.0, 3.0].to_vec()),
            [0.2672612419124244, 0.5345224838248488, 0.8017837257372732].to_vec()
        );
    }

    #[pg_test]
    fn test_normalize_max_s() {
        assert_eq!(
            normalize_max_s([1.0, 2.0, 3.0].to_vec()),
            [0.33333334, 0.6666667, 1.0].to_vec()
        );
    }

    #[pg_test]
    fn test_normalize_max_d() {
        assert_eq!(
            normalize_max_d([1.0, 2.0, 3.0].to_vec()),
            [0.3333333333333333, 0.6666666666666666, 1.0].to_vec()
        );
    }

    #[pg_test]
    fn test_distance_l1_s() {
        assert_eq!(
            distance_l1_s([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            0.0
        );
    }

    #[pg_test]
    fn test_distance_l1_d() {
        assert_eq!(
            distance_l1_d([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            0.0
        );
    }

    #[pg_test]
    fn test_distance_l2_s() {
        assert_eq!(
            distance_l2_s([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            0.0
        );
    }

    #[pg_test]
    fn test_distance_l2_d() {
        assert_eq!(
            distance_l2_d([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            0.0
        );
    }

    #[pg_test]
    fn test_dot_product_s() {
        assert_eq!(
            dot_product_s([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            14.0
        );
        assert_eq!(
            dot_product_s([1.0, 2.0, 3.0].to_vec(), [2.0, 3.0, 4.0].to_vec()),
            20.0
        );
    }

    #[pg_test]
    fn test_dot_product_d() {
        assert_eq!(
            dot_product_d([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            14.0
        );
        assert_eq!(
            dot_product_d([1.0, 2.0, 3.0].to_vec(), [2.0, 3.0, 4.0].to_vec()),
            20.0
        );
    }

    #[pg_test]
    fn test_cosine_similarity_s() {
        assert_eq!(
            cosine_similarity_s([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            0.99999994
        );
        assert_eq!(
            cosine_similarity_s([1.0, 2.0, 3.0].to_vec(), [2.0, 3.0, 4.0].to_vec()),
            0.9925833
        );
    }

    #[pg_test]
    fn test_cosine_similarity_d() {
        assert_eq!(
            cosine_similarity_d([1.0, 2.0, 3.0].to_vec(), [1.0, 2.0, 3.0].to_vec()),
            1.0
        );
        assert_eq!(
            cosine_similarity_d([1.0, 2.0, 3.0].to_vec(), [2.0, 3.0, 4.0].to_vec()),
            0.9925833339709303
        );
    }
}
