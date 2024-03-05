use lazy_static::lazy_static;
use pgrx::{GucContext, GucFlags, GucRegistry, GucSetting};
use std::ffi::CStr;

#[cfg(any(test, feature = "pg_test"))]
use pgrx::{pg_schema, pg_test};

lazy_static! {
    pub static ref PGML_VENV: (&'static str, GucSetting<Option<&'static CStr>>) =
        ("pgml.venv", GucSetting::<Option<&'static CStr>>::new(None));
    pub static ref PGML_HF_WHITELIST: (&'static str, GucSetting<Option<&'static CStr>>) = (
        "pgml.huggingface_whitelist",
        GucSetting::<Option<&'static CStr>>::new(None),
    );
    pub static ref PGML_HF_TRUST_REMOTE_CODE: (&'static str, GucSetting<bool>) =
        ("pgml.huggingface_trust_remote_code", GucSetting::<bool>::new(false));
    pub static ref PGML_HF_TRUST_WHITELIST: (&'static str, GucSetting<Option<&'static CStr>>) = (
        "pgml.huggingface_trust_remote_code_whitelist",
        GucSetting::<Option<&'static CStr>>::new(None),
    );
}

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
}
