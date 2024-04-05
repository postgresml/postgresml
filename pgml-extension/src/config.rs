use once_cell::sync::Lazy;
use pgrx::{GucContext, GucFlags, GucRegistry, GucSetting};
use std::ffi::CStr;

#[cfg(any(test, feature = "pg_test"))]
use pgrx::{pg_schema, pg_test};

pub static PGML_VENV: Lazy<(&'static str, GucSetting<Option<&'static CStr>>)> =
    Lazy::new(|| ("pgml.venv", GucSetting::<Option<&'static CStr>>::new(None)));
pub static PGML_HF_WHITELIST: Lazy<(&'static str, GucSetting<Option<&'static CStr>>)> = Lazy::new(|| {
    (
        "pgml.huggingface_whitelist",
        GucSetting::<Option<&'static CStr>>::new(None),
    )
});
pub static PGML_HF_TRUST_REMOTE_CODE: Lazy<(&'static str, GucSetting<bool>)> =
    Lazy::new(|| ("pgml.huggingface_trust_remote_code", GucSetting::<bool>::new(false)));
pub static PGML_HF_TRUST_WHITELIST: Lazy<(&'static str, GucSetting<Option<&'static CStr>>)> = Lazy::new(|| {
    (
        "pgml.huggingface_trust_remote_code_whitelist",
        GucSetting::<Option<&'static CStr>>::new(None),
    )
});
pub static PGML_OMP_NUM_THREADS: Lazy<(&'static str, GucSetting<i32>)> =
    Lazy::new(|| ("pgml.omp_num_threads", GucSetting::<i32>::new(0)));

pub fn initialize_server_params() {
    GucRegistry::define_string_guc(
        PGML_VENV.0,
        "Python's virtual environment path",
        "",
        &PGML_VENV.1,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_string_guc(
        PGML_HF_WHITELIST.0,
        "Models allowed to be downloaded from huggingface",
        "",
        &PGML_HF_WHITELIST.1,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        PGML_HF_TRUST_REMOTE_CODE.0,
        "Whether model can execute remote codes",
        "",
        &PGML_HF_TRUST_REMOTE_CODE.1,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_string_guc(
        PGML_HF_TRUST_WHITELIST.0,
        "Models allowed to execute remote codes when pgml.hugging_face_trust_remote_code = 'on'",
        "",
        &PGML_HF_TRUST_WHITELIST.1,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        PGML_OMP_NUM_THREADS.0,
        "Specifies the number of threads used by default of underlying OpenMP library. Only positive integers are valid",
        "",
        &PGML_OMP_NUM_THREADS.1,
        0,
        i32::max_value(),
        GucContext::Backend,
        GucFlags::default(),
    );
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
    fn read_pgml_huggingface_whitelist() {
        let name = "pgml.huggingface_whitelist";
        let value = "meta-llama/Llama-2-7b";
        set_config(name, value).unwrap();
        assert_eq!(PGML_HF_WHITELIST.1.get().unwrap().to_string_lossy(), value);
    }

    #[pg_test]
    fn omp_num_threads_cannot_be_set_after_startup() {
        let result = std::panic::catch_unwind(|| set_config("pgml.omp_num_threads", "1"));
        assert!(result.is_err());
    }
}
