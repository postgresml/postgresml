use pgrx::{GucContext, GucFlags, GucRegistry, GucSetting};
use std::ffi::CStr;

#[cfg(any(test, feature = "pg_test"))]
use pgrx::{pg_schema, pg_test};

pub static PGML_VENV: GucSetting<Option<&'static CStr>> = GucSetting::<Option<&'static CStr>>::new(None);
pub static PGML_HF_WHITELIST: GucSetting<Option<&'static CStr>> = GucSetting::<Option<&'static CStr>>::new(None);
pub static PGML_HF_TRUST_REMOTE_CODE: GucSetting<bool> = GucSetting::<bool>::new(false);
pub static PGML_HF_TRUST_REMOTE_CODE_WHITELIST: GucSetting<Option<&'static CStr>> =
    GucSetting::<Option<&'static CStr>>::new(None);
pub static PGML_OMP_NUM_THREADS: GucSetting<i32> = GucSetting::<i32>::new(1);

extern "C" {
    fn omp_set_num_threads(num_threads: i32);
}

pub fn initialize_server_params() {
    GucRegistry::define_string_guc(
        "pgml.venv",
        "Python's virtual environment path",
        "",
        &PGML_VENV,
        GucContext::Userset,
        GucFlags::default(),
    );

    GucRegistry::define_string_guc(
        "pgml.huggingface_whitelist",
        "Models allowed to be downloaded from huggingface",
        "",
        &PGML_HF_WHITELIST,
        GucContext::Userset,
        GucFlags::default(),
    );

    GucRegistry::define_bool_guc(
        "pgml.huggingface_trust_remote_code",
        "Whether model can execute remote codes",
        "",
        &PGML_HF_TRUST_REMOTE_CODE,
        GucContext::Userset,
        GucFlags::default(),
    );

    GucRegistry::define_string_guc(
        "pgml.huggingface_trust_remote_code_whitelist",
        "Models allowed to execute remote codes when pgml.hugging_face_trust_remote_code = 'on'",
        "",
        &PGML_HF_TRUST_REMOTE_CODE_WHITELIST,
        GucContext::Userset,
        GucFlags::default(),
    );

    GucRegistry::define_int_guc(
        "pgml.omp_num_threads",
        "Specifies the number of threads used by default of underlying OpenMP library. Only positive integers are valid",
        "",
        &PGML_OMP_NUM_THREADS,
        1,
        i32::max_value(),
        GucContext::Backend,
        GucFlags::default(),
    );

    let omp_num_threads = PGML_OMP_NUM_THREADS.get();
    unsafe {
        omp_set_num_threads(omp_num_threads);
    }
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
        assert_eq!(PGML_HF_WHITELIST.get().unwrap().to_str().unwrap(), value);
    }

    #[pg_test]
    fn omp_num_threads_cannot_be_set_after_startup() {
        let result = std::panic::catch_unwind(|| set_config("pgml.omp_num_threads", "1"));
        assert!(result.is_err());
    }
}
